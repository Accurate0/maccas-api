use http::Method;
use lambda_http::request::RequestContext;
use lambda_http::{service_fn, Error, IntoResponse, Request, RequestExt, Response};

pub mod api;
pub mod client;
pub mod types;

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_http::run(service_fn(run)).await?;
    Ok(())
}

async fn run(request: Request) -> Result<impl IntoResponse, Error> {
    let api_client = client::get().await?;
    let params = request.path_parameters();
    let context = request.request_context();

    let resource_path = match context {
        RequestContext::ApiGatewayV1(r) => r.resource_path,
        _ => panic!(),
    };

    Ok(match resource_path {
        Some(s) => match s.as_str() {
            "/deals" => {
                let resp = api_client
                    .get_offers(None)
                    .await?
                    .response
                    .expect("to have response")
                    .offers;

                serde_json::to_string(&resp).unwrap().into_response()
            }

            "/deals/{dealId}" => {
                let deal_id = params.first("dealId").expect("must have id");
                let deal_id = &deal_id.to_owned();

                match *request.method() {
                    Method::POST => {
                        let resp = api_client
                            .add_offer_to_offers_dealstack(deal_id, None, None)
                            .await?;

                        serde_json::to_string(&resp).unwrap().into_response()
                    }

                    Method::DELETE => {
                        let resp = api_client
                            .get_offers(None)
                            .await?
                            .response
                            .expect("to have response")
                            .offers;

                        let offer_id_vec: Vec<i64> = resp
                            .iter()
                            .filter(|d| d.offer_proposition_id.to_string() == *deal_id)
                            .map(|d| d.offer_id)
                            .collect();

                        let offer_id = offer_id_vec.first().unwrap();

                        let resp = api_client
                            .remove_offer_from_offers_dealstack(*offer_id, deal_id, None, None)
                            .await?;

                        serde_json::to_string(&resp).unwrap().into_response()
                    }

                    _ => panic!(),
                }
            }

            _ => Response::builder()
                .status(400)
                .body("Bad Request".into())
                .expect("failed to render response"),
        },
        None => Response::builder()
            .status(400)
            .body("Bad Request".into())
            .expect("failed to render response"),
    })
}

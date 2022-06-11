use crate::cache;
use crate::dispatcher::Executor;
use crate::{client, config::ApiConfig};
use async_trait::async_trait;
use http::Response;
use lambda_http::{Body, Error, IntoResponse, Request, RequestExt};

pub struct Code;

#[async_trait]
impl Executor for Code {
    async fn execute(
        &self,
        request: &Request,
        dynamodb_client: &aws_sdk_dynamodb::Client,
        config: &ApiConfig,
    ) -> Result<Response<Body>, Error> {
        let path_params = request.path_parameters();
        let query_params = request.query_string_parameters();

        let store = query_params.first("store");
        let deal_id = path_params.first("dealId").expect("must have id");
        let deal_id = &deal_id.to_owned();

        let (account_name, _offer) =
            cache::get_offer_by_id(deal_id, &dynamodb_client, &config.cache_table_name_v2).await?;
        let user = config
            .users
            .iter()
            .find(|u| u.account_name == account_name)
            .unwrap();

        let http_client = client::get_http_client();
        let api_client = client::get(
            &http_client,
            &dynamodb_client,
            &account_name,
            &config,
            &user.login_username,
            &user.login_password,
        )
        .await?;

        let resp = api_client.offers_dealstack(None, store).await?;
        Ok(serde_json::to_string(&resp).unwrap().into_response())
    }
}

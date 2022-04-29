use crate::Bot;
use serenity::client::Context;
use serenity::model::interactions::application_command::ApplicationCommandInteraction;
use serenity::model::interactions::InteractionResponseType;
use types::maccas;

impl Bot {
    pub async fn code_command(&self, ctx: &Context, command: &ApplicationCommandInteraction) {
        command
            .create_interaction_response(&ctx.http, |r| {
                r.kind(InteractionResponseType::DeferredChannelMessageWithSource)
                    .interaction_response_data(|d| d.ephemeral(true))
            })
            .await
            .unwrap();

        let deal_id = command
            .data
            .options
            .get(0)
            .expect("Expected option")
            .value
            .as_ref()
            .expect("Expected string")
            .as_str()
            .unwrap();

        let resp = self
            .api_client
            .request_without_deserialize(http::Method::GET, format!("code/{deal_id}").as_str())
            .await;

        match resp.status() {
            reqwest::StatusCode::OK => {
                let resp = resp.json::<maccas::OfferDealStackResponse>().await.unwrap();
                let code = resp.response.unwrap().random_code;

                command
                    .edit_original_interaction_response(&ctx, |m| {
                        m.embed(|e| {
                            e.colour(0xDA291C as i32)
                                .title("Code")
                                .description(code)
                                .field("Response", format!("{}", resp.status.message), false)
                        })
                    })
                    .await
                    .unwrap();
            }
            _ => {
                command
                    .edit_original_interaction_response(&ctx, |m| {
                        m.embed(|e| {
                            e.colour(0xDA291C as i32)
                                .title("Response")
                                .description(resp.status())
                        })
                    })
                    .await
                    .unwrap();
            }
        }
    }
}

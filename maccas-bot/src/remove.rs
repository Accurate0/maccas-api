use crate::{constants, Bot};
use http::Method;
use serenity::client::Context;
use serenity::model::interactions::application_command::ApplicationCommandInteraction;
use serenity::model::interactions::InteractionResponseType;

impl Bot {
    pub async fn remove_command(
        &self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
        is_public: bool,
    ) {
        command
            .create_interaction_response(&ctx.http, |r| {
                r.kind(InteractionResponseType::DeferredChannelMessageWithSource)
                    .interaction_response_data(|d| d.ephemeral(is_public))
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
            .maccas_request_without_deserialize(Method::DELETE, format!("deals/{deal_id}").as_str())
            .await
            .status();

        command
            .edit_original_interaction_response(&ctx, |m| {
                m.embed(|e| {
                    e.colour(constants::MACCAS_RED)
                        .title("Response")
                        .description(resp)
                })
            })
            .await
            .unwrap();
    }
}

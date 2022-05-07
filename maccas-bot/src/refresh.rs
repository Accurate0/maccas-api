use crate::{constants, Bot};
use serenity::client::Context;
use serenity::model::interactions::application_command::ApplicationCommandInteraction;
use serenity::model::interactions::InteractionResponseType;

impl Bot {
    pub async fn refresh_command(
        &self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
        _is_public: bool,
    ) {
        command
            .create_interaction_response(&ctx.http, |r| {
                r.kind(InteractionResponseType::DeferredChannelMessageWithSource)
            })
            .await
            .unwrap();

        let resp = self
            .api_client
            .maccas_request_without_deserialize(http::Method::POST, "deals/refresh")
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

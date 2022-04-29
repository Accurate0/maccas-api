use crate::Bot;
use serenity::client::Context;
use serenity::model::interactions::application_command::ApplicationCommandInteraction;
use serenity::model::interactions::InteractionResponseType;

impl Bot {
    pub async fn remove_command(&self, ctx: &Context, command: &ApplicationCommandInteraction) {
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

        let url = &self
            .base_url
            .join(format!("deals/{deal_id}").as_str())
            .unwrap();

        let resp = self
            .client
            .delete(url.as_str())
            .send()
            .await
            .unwrap()
            .status();

        command
            .edit_original_interaction_response(&ctx, |m| {
                m.embed(|e| {
                    e.colour(0xDA291C as i32)
                        .title("Response")
                        .description(resp)
                })
            })
            .await
            .unwrap();
    }
}

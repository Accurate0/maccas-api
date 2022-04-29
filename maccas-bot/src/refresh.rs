use crate::Bot;
use serenity::client::Context;
use serenity::model::interactions::application_command::ApplicationCommandInteraction;
use serenity::model::interactions::InteractionResponseType;

impl Bot {
    pub async fn refresh_command(&self, ctx: &Context, command: &ApplicationCommandInteraction) {
        command
            .create_interaction_response(&ctx.http, |r| {
                r.kind(InteractionResponseType::DeferredChannelMessageWithSource)
            })
            .await
            .unwrap();

        let url = &self.base_url.join("deals/refresh").unwrap();

        let resp = self
            .client
            .post(url.as_str())
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

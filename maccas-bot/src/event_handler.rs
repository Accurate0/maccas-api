use crate::Bot;
use serenity::async_trait;
use serenity::client::{Context, EventHandler};
use serenity::model::gateway::Ready;
use serenity::model::interactions::application_command::{
    ApplicationCommand, ApplicationCommandOptionType,
};
use serenity::model::interactions::Interaction;

#[async_trait]
impl EventHandler for Bot {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let is_public = match command.channel_id.to_channel(&ctx).await {
                Ok(c) => match c.private() {
                    Some(_) => false,
                    None => true,
                },
                _ => true,
            };

            #[rustfmt::skip]
            match command.data.name.as_str() {
                "refresh"    => self.refresh_command(&ctx, &command, is_public).await,
                "remove"     => self.remove_command(&ctx, &command, is_public).await,
                "code"       => self.code_command(&ctx, &command, is_public).await,
                "deals"      => self.deals_command(&ctx, &command, is_public).await,
                "location"   => self.location_command(&ctx, &command, is_public).await,
                _ => panic!(),
            };
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        log::warn!("{} is connected!", ready.user.name);
        ApplicationCommand::create_global_application_command(&ctx.http, |command| {
            command.name("deals").description("fuck elliot walker")
        })
        .await
        .unwrap();

        ApplicationCommand::create_global_application_command(&ctx.http, |command| {
            command
                .name("remove")
                .description("remove a deal")
                .create_option(|opt| {
                    opt.name("deal_id")
                        .description("the deal id")
                        .kind(ApplicationCommandOptionType::String)
                        .required(true)
                })
        })
        .await
        .unwrap();

        ApplicationCommand::create_global_application_command(&ctx.http, |command| {
            command
                .name("code")
                .description("get code for existing deal")
                .create_option(|opt| {
                    opt.name("deal_id")
                        .description("the deal id")
                        .kind(ApplicationCommandOptionType::String)
                        .required(true)
                })
        })
        .await
        .unwrap();

        ApplicationCommand::create_global_application_command(&ctx.http, |command| {
            command
                .name("location")
                .description("set your location for nearest maccas")
                .create_option(|opt| {
                    opt.name("location")
                        .description("location to lookup")
                        .kind(ApplicationCommandOptionType::String)
                        .required(true)
                })
        })
        .await
        .unwrap();

        ApplicationCommand::create_global_application_command(&ctx.http, |command| {
            command
                .name("refresh")
                .description("force refresh deal list")
        })
        .await
        .unwrap();
    }
}

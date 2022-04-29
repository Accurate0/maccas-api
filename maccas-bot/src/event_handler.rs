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
            #[rustfmt::skip]
            match command.data.name.as_str() {
                "refresh" => self.refresh_command(&ctx, &command).await,
                "remove"  => self.remove_command(&ctx, &command).await,
                "code"    => self.code_command(&ctx, &command).await,
                "deals"   => self.deals_command(&ctx, &command).await,
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
                .name("refresh")
                .description("force refresh deal list")
        })
        .await
        .unwrap();
    }
}

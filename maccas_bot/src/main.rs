use config::Config;
use log::*;
use reqwest::header;
use simplelog::*;
use std::time::Duration;

use serenity::async_trait;
use serenity::builder::{CreateActionRow, CreateSelectMenu, CreateSelectMenuOption};
use serenity::client::{Context, EventHandler};
use serenity::model::interactions::application_command::{
    ApplicationCommand, ApplicationCommandOptionType,
};
use serenity::model::interactions::InteractionResponseType;
use serenity::prelude::*;

use serenity::model::gateway::Ready;

use libmaccas::types;
use serenity::model::interactions::Interaction;

struct Bot {
    client: reqwest::Client,
    base_url: reqwest::Url,
}

fn setup_logging() {
    let term_config = ConfigBuilder::new()
        .set_level_padding(LevelPadding::Right)
        .build();

    TermLogger::init(
        LevelFilter::Info,
        term_config,
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();
}

#[async_trait]
impl EventHandler for Bot {
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
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            match command.data.name.as_str() {
                "remove" => {
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

                "code" => {
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
                        .join(format!("code/{deal_id}").as_str())
                        .unwrap();

                    let resp = self.client.get(url.as_str()).send().await.unwrap();

                    match resp.status() {
                        reqwest::StatusCode::OK => {
                            let resp = resp.json::<types::OfferDealStackResponse>().await.unwrap();
                            let code = resp.response.unwrap().random_code;

                            command
                                .edit_original_interaction_response(&ctx, |m| {
                                    m.embed(|e| {
                                        e.colour(0xDA291C as i32)
                                            .title("Code")
                                            .description(code)
                                            .field(
                                                "Response",
                                                format!("{}", resp.status.message),
                                                false,
                                            )
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

                "deals" => {
                    command
                        .create_interaction_response(&ctx.http, |r| {
                            r.kind(InteractionResponseType::DeferredChannelMessageWithSource)
                                .interaction_response_data(|d| d.ephemeral(true))
                        })
                        .await
                        .unwrap();

                    let url = &self.base_url.join("deals").unwrap();
                    let resp = self
                        .client
                        .get(url.as_str())
                        .send()
                        .await
                        .unwrap()
                        .json::<Vec<types::Offer>>()
                        .await
                        .unwrap();

                    let mut menu = CreateSelectMenu::default();
                    menu.custom_id("fuck-elliot-walker");
                    menu.placeholder("No offer selected");

                    let options = resp
                        .iter()
                        .filter(|offer| offer.offer_id != 0)
                        .map(|offer| {
                            let mut opt = CreateSelectMenuOption::default();

                            let cloned_name = offer.name.clone();
                            let split: Vec<&str> = cloned_name.split("\n").collect();

                            opt.label(split[0]);
                            opt.value(offer.offer_id);

                            opt
                        });

                    menu.options(|f| {
                        for option in options {
                            f.add_option(option);
                        }
                        f
                    });

                    let mut ar = CreateActionRow::default();
                    ar.add_select_menu(menu);

                    let message = command
                        .edit_original_interaction_response(&ctx.http, |m| {
                            m.components(|c| c.add_action_row(ar.clone()))
                        })
                        .await
                        .unwrap();

                    let mci = match message
                        .await_component_interaction(&ctx)
                        .timeout(Duration::from_secs(180))
                        .await
                    {
                        Some(ci) => ci,
                        None => {
                            message.reply(&ctx, "Timed out").await.unwrap();
                            return;
                        }
                    };

                    let offer_id = mci.data.values.get(0).unwrap();

                    mci.create_interaction_response(&ctx, |r| {
                        r.kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|d| {
                                d.ephemeral(true)
                                    .content(format!("You chose: **{}**", offer_id))
                            })
                    })
                    .await
                    .unwrap();

                    let url = &self
                        .base_url
                        .join(format!("deals/{offer_id}").as_str())
                        .unwrap();

                    let resp = self
                        .client
                        .post(url.as_str())
                        .send()
                        .await
                        .unwrap()
                        .json::<types::OfferDealStackResponse>()
                        .await
                        .unwrap();

                    let code = match resp.response {
                        Some(r) => r.random_code,
                        None => {
                            let url = &self
                                .base_url
                                .join(format!("code/{offer_id}").as_str())
                                .unwrap();

                            let resp = self
                                .client
                                .get(url.as_str())
                                .send()
                                .await
                                .unwrap()
                                .json::<types::OfferDealStackResponse>()
                                .await
                                .unwrap();

                            resp.response.unwrap().random_code
                        }
                    };

                    mci.edit_original_interaction_response(&ctx, |m| {
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
                _ => panic!(),
            };
        }
    }
}

#[derive(serde::Deserialize, std::fmt::Debug)]
#[serde(rename_all = "camelCase")]
struct BotConfig {
    pub api_key: String,
    pub base_url: String,
    pub discord_token: String,
}

#[tokio::main]
async fn main() {
    setup_logging();

    let config = Config::builder()
        .add_source(config::File::from_str(
            std::include_str!("../config.yml"),
            config::FileFormat::Yaml,
        ))
        .build()
        .unwrap()
        .try_deserialize::<BotConfig>()
        .expect("valid configuration present");

    let mut api_key_header = header::HeaderValue::from_str(config.api_key.as_str()).unwrap();
    api_key_header.set_sensitive(true);

    let mut headers = header::HeaderMap::new();
    headers.insert("X-Api-Key", api_key_header);
    headers.insert("Content-Length", header::HeaderValue::from(0 as i32));

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();

    let base_url = reqwest::Url::parse(&config.base_url.as_str()).unwrap();
    let bot = Bot { client, base_url };

    let mut client = Client::builder(
        config.discord_token,
        GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::DIRECT_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT,
    )
    .event_handler(bot)
    .await
    .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

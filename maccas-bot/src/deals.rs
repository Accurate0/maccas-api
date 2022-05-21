use crate::{constants, Bot};
use chrono::Local;
use chrono::{DateTime, Utc};
use http::Method;
use serenity::builder::{CreateActionRow, CreateSelectMenu, CreateSelectMenuOption};
use serenity::client::Context;
use serenity::model::interactions::application_command::ApplicationCommandInteraction;
use serenity::model::interactions::InteractionResponseType;
use std::time::Duration;
use std::time::SystemTime;
use types::api::Offer;
use types::bot::UsageLog;
use types::bot::UserOptions;
use types::maccas;

impl Bot {
    pub async fn deals_command(
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

        let resp = self
            .api_client
            .maccas_request::<Vec<Offer>>(Method::GET, "deals")
            .await;

        let now = SystemTime::now();
        let now: DateTime<Utc> = now.into();
        let now = now.timestamp();

        let mut deals_to_lock = Vec::<String>::new();
        let options: Vec<CreateSelectMenuOption> = resp
            .clone()
            .into_iter()
            .map(|offer| {
                let mut opt = CreateSelectMenuOption::default();

                let cloned_name = offer.name.clone();
                let split: Vec<&str> = cloned_name.split("\n").collect();
                let valid_time_from = DateTime::parse_from_rfc3339(&offer.valid_from_utc)
                    .unwrap()
                    .timestamp();
                let valid_time_to = DateTime::parse_from_rfc3339(&offer.valid_to_utc)
                    .unwrap()
                    .timestamp();
                let emoji = if valid_time_from < now && valid_time_to > now {
                    "✅"
                } else {
                    "❌"
                };
                let offer_name = String::from(split[0]);
                let uuid = offer.deal_uuid;
                let count = offer.count;

                opt.label(format!("{emoji} {offer_name} ({count})"));
                opt.value(&uuid);
                deals_to_lock.push(String::from(uuid));

                opt
            })
            .collect();

        let mut x = 0 as u8;
        let mut ars = Vec::<CreateActionRow>::new();
        for chunk in options.chunks(25).into_iter() {
            let mut ar = CreateActionRow::default();
            let mut menu = CreateSelectMenu::default();
            menu.custom_id(x.to_string());
            menu.placeholder("No offer selected");
            menu.options(|f| {
                for option in chunk {
                    f.add_option(option.clone());
                }
                f
            });
            ar.add_select_menu(menu);
            x += 1;
            ars.push(ar);
        }

        // lock these deals for 120 seconds...
        const DURATION: u64 = 120;
        for deal in &deals_to_lock {
            self.api_client
                .maccas_request_without_deserialize(
                    Method::POST,
                    format!("deals/lock/{deal}?duration={DURATION}").as_str(),
                )
                .await;
        }

        let message = command
            .create_followup_message(&ctx.http, |m| {
                m.components(|c| {
                    for ar in ars {
                        c.add_action_row(ar.clone());
                    }
                    c
                })
            })
            .await
            .unwrap();

        let mci = match message
            .await_component_interaction(&ctx)
            .timeout(Duration::from_secs(DURATION))
            .await
        {
            Some(ci) => ci,
            None => {
                command
                    .edit_original_interaction_response(&ctx.http, |m| {
                        m.content("Timed out").components(|c| c)
                    })
                    .await
                    .unwrap();

                for deal in &deals_to_lock {
                    self.api_client
                        .maccas_request_without_deserialize(
                            Method::DELETE,
                            format!("deals/lock/{deal}").as_str(),
                        )
                        .await;
                }

                return;
            }
        };

        let offer_id = mci.data.values.get(0).unwrap();
        let offer_name = resp
            .iter()
            .find(|offer| offer.deal_uuid == *offer_id)
            .unwrap()
            .name
            .clone();
        let offer_name = offer_name.split("\n").collect::<Vec<&str>>();
        let offer_name = offer_name[0];

        mci.create_interaction_response(&ctx, |r| {
            r.kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|d| {
                    d.ephemeral(is_public)
                        .content(format!("You chose: **{}**", offer_name))
                })
        })
        .await
        .unwrap();

        let user_id = command.user.id.as_u64().to_string();
        let user_options = self.api_client.kvp_get(&user_id).await;
        let store_id = match user_options.status() {
            reqwest::StatusCode::OK => {
                let json = user_options.json::<UserOptions>().await.unwrap();
                Some(json.store_id)
            }
            _ => None,
        };

        let endpoint = match store_id {
            Some(s) => format!("deals/{offer_id}?store={s}"),
            None => format!("deals/{offer_id}"),
        };

        let resp = self
            .api_client
            .maccas_request::<maccas::OfferDealStackResponse>(Method::POST, endpoint.as_str())
            .await;

        let code = match resp.response {
            Some(r) => r.random_code,
            None => {
                let resp = self
                    .api_client
                    .maccas_request::<maccas::OfferDealStackResponse>(
                        Method::GET,
                        format!("code/{offer_id}").as_str(),
                    )
                    .await;

                resp.response.unwrap().random_code
            }
        };

        mci.edit_original_interaction_response(&ctx, |m| {
            m.embed(|e| {
                e.colour(constants::MACCAS_RED)
                    .title("Code")
                    .description(code)
                    .field("Offer ID", format!("{}", offer_id), false)
                    .field("Response", format!("{}", resp.status.message), false)
            })
        })
        .await
        .unwrap();

        // easy to copy paste on mobile
        mci.create_followup_message(&ctx, |m| m.ephemeral(is_public).content(offer_id))
            .await
            .unwrap();

        // unlock
        for deal in &deals_to_lock {
            if deal == offer_id {
                continue;
            }

            self.api_client
                .maccas_request_without_deserialize(
                    Method::DELETE,
                    format!("deals/lock/{deal}").as_str(),
                )
                .await;
        }

        command
            .edit_original_interaction_response(&ctx.http, |m| {
                m.content("Interaction finished.").components(|c| c)
            })
            .await
            .unwrap();

        let dt: DateTime<Local> = Local::now();

        // log this request
        let usage_log = UsageLog {
            user_id,
            deal_readable: offer_name.to_string(),
            deal_uuid: offer_id.to_string(),
            user_readable: command.user.name.to_string(),
            message: "Deal Used",
            local_time: dt.format("%a %b %e %T %Y").to_string(),
        };
        self.api_client.log(&usage_log).await
    }
}

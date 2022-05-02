use crate::{constants, Bot};
use http::Method;
use itertools::Itertools;
use serenity::builder::{CreateActionRow, CreateSelectMenu, CreateSelectMenuOption};
use serenity::client::Context;
use serenity::model::interactions::application_command::ApplicationCommandInteraction;
use serenity::model::interactions::InteractionResponseType;
use std::time::Duration;
use types::maccas;

impl Bot {
    pub async fn deals_command(&self, ctx: &Context, command: &ApplicationCommandInteraction) {
        command
            .create_interaction_response(&ctx.http, |r| {
                r.kind(InteractionResponseType::DeferredChannelMessageWithSource)
                    .interaction_response_data(|d| d.ephemeral(true))
            })
            .await
            .unwrap();

        let resp = self
            .api_client
            .maccas_request::<Vec<maccas::Offer>>(Method::GET, "deals")
            .await;

        let mut deals_to_lock = Vec::<String>::new();
        let options: Vec<CreateSelectMenuOption> = resp
            .into_iter()
            .unique_by(|offer| offer.offer_proposition_id)
            .map(|offer| {
                let mut opt = CreateSelectMenuOption::default();

                let cloned_name = offer.name.clone();
                let split: Vec<&str> = cloned_name.split("\n").collect();

                let uuid = offer.deal_uuid.unwrap();
                opt.label(split[0]);
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

        // lock these deals for 180 seconds...
        const DURATION: u64 = 180;
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

        mci.create_interaction_response(&ctx, |r| {
            r.kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|d| {
                    d.ephemeral(true)
                        .content(format!("You chose: **{}**", offer_id))
                })
        })
        .await
        .unwrap();

        let resp = self
            .api_client
            .maccas_request::<maccas::OfferDealStackResponse>(
                Method::POST,
                format!("deals/{offer_id}").as_str(),
            )
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
                    .field("Response", format!("{}", resp.status.message), false)
            })
        })
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
            .edit_original_interaction_response(&ctx.http, |m| m.content("Interaction finished."))
            .await
            .unwrap();
    }
}

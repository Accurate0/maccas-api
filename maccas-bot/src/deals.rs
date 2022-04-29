use crate::{constants, Bot};
use http::Method;
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

        let options: Vec<CreateSelectMenuOption> = resp
            .iter()
            // 0 "can't" be selected in App
            // 30762 is McCafé®, Buy 5 Get 1 Free, valid till end of year...
            .filter(|offer| offer.offer_id != 0 && offer.offer_proposition_id != 30762)
            .map(|offer| {
                let mut opt = CreateSelectMenuOption::default();

                let cloned_name = offer.name.clone();
                let split: Vec<&str> = cloned_name.split("\n").collect();

                opt.label(split[0]);
                opt.value(offer.offer_id);

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

        let message = command
            .edit_original_interaction_response(&ctx.http, |m| {
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

        let resp = self
            .api_client
            .maccas_request::<maccas::OfferDealStackResponse>(
                Method::POST,
                format!("deals/{offer_id}").as_str(),
            )
            .await;

        // Lock deal for 15 minutes, to prevent it from showing in GET /deals requests
        self.api_client
            .maccas_request_without_deserialize(
                Method::POST,
                format!("deals/lock/{offer_id}").as_str(),
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
    }
}

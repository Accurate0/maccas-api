use std::time::Duration;

use serenity::builder::{CreateActionRow, CreateSelectMenu, CreateSelectMenuOption};
use serenity::client::Context;
use serenity::model::interactions::application_command::ApplicationCommandInteraction;
use serenity::model::interactions::InteractionResponseType;
use types::maccas;

use crate::Bot;

impl Bot {
    pub async fn deals_command(&self, ctx: &Context, command: &ApplicationCommandInteraction) {
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
            .json::<Vec<maccas::Offer>>()
            .await
            .unwrap();

        let options: Vec<CreateSelectMenuOption> = resp
            .iter()
            .filter(|offer| offer.offer_id != 0)
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
            .json::<maccas::OfferDealStackResponse>()
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
                    .json::<maccas::OfferDealStackResponse>()
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
}

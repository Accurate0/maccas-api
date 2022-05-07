use crate::{constants, Bot};
use http::Method;
use serenity::client::Context;
use serenity::model::interactions::application_command::ApplicationCommandInteraction;
use serenity::model::interactions::InteractionResponseType;
use types::bot::UserOptions;
use types::maccas::RestaurantLocationResponse;

impl Bot {
    pub async fn location_command(
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

        let location = command
            .data
            .options
            .get(0)
            .expect("Expected option")
            .value
            .as_ref()
            .expect("Expected string")
            .as_str()
            .unwrap();

        let resp = self.api_client.place_request(location).await;
        let location = resp.result.unwrap().geometry.location;
        let lat = location.lat;
        let lng = location.lng;

        const DISTANCE: u64 = 500;

        let endpoint = format!("locations?distance={DISTANCE}&latitude={lat}&longitude={lng}");

        let resp = self
            .api_client
            .maccas_request::<RestaurantLocationResponse>(Method::GET, endpoint.as_str())
            .await;

        let response = resp.response.unwrap();
        let selected_restaurant = response.restaurants.first().unwrap();
        let store_id = selected_restaurant.national_store_number.to_string();

        let user_id = command.user.id.as_u64().to_string();
        let user_options = UserOptions { store_id };

        let resp = self.api_client.kvp_set(&user_id, &user_options).await;
        dbg!(resp);

        command
            .edit_original_interaction_response(&ctx, |m| {
                m.embed(|e| {
                    e.colour(constants::MACCAS_RED)
                        .title("Store Selected")
                        .description(&selected_restaurant.address.address_line1)
                })
            })
            .await
            .unwrap();
    }
}

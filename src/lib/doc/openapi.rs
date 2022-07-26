use crate::routes::code::get_code::*;
use crate::routes::deal::get_deal::*;
use crate::routes::deals::add_remove::*;
use crate::routes::deals::get_deals::*;
use crate::routes::deals::last_refresh::*;
use crate::routes::locations::get_locations::*;
use crate::routes::locations::search::*;
use crate::routes::points::get_by_id::*;
use crate::routes::points::get_points::*;
use crate::routes::statistics::account::*;
use crate::routes::statistics::total_accounts::*;
use crate::routes::user::config::*;
use crate::types::api::{
    AccountPointMap, AccountPointResponse, AccountResponse, LastRefreshInformation, Offer,
    OfferResponse, RestaurantAddress, RestaurantInformation,
};
use crate::types::user::UserOptions;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    handlers(
        get_code,
        get_deals,
        add_deal,
        remove_deal,
        get_deal,
        get_locations,
        last_refresh,
        search_locations,
        get_points_by_id,
        get_points,
        get_accounts,
        get_total_accounts,
        get_user_config,
        update_user_config,
    ),
    components(
        Offer,
        OfferResponse,
        LastRefreshInformation,
        RestaurantInformation,
        AccountResponse,
        UserOptions,
        RestaurantAddress,
        AccountPointResponse,
        AccountPointMap,
    )
)]
pub struct ApiDoc;

use crate::routes::code::get_code_docs::*;
use crate::routes::deal::docs::*;
use crate::routes::deals::add_remove_docs::*;
use crate::routes::deals::get_deals_docs::*;
use crate::routes::deals::last_refresh_docs::*;
use crate::routes::deals::lock_docs::*;
use crate::routes::locations::get_locations_docs::*;
use crate::routes::locations::search_locations_docs::*;
use crate::routes::points::get_by_id_docs::*;
use crate::routes::points::get_points_docs::*;
use crate::routes::statistics::account_docs::*;
use crate::routes::statistics::total_accounts_docs::*;
use crate::routes::user::config_docs::*;
use crate::types::api::{
    AccountPointMap, AccountPointResponse, AccountResponse, Error, LastRefreshInformation, Offer, OfferResponse,
    RestaurantAddress, RestaurantInformation,
};
use crate::types::user::UserOptions;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    handlers(
        deals,
        points,
        get_deal,
        add_deal,
        get_code,
        remove_deal,
        get_points_by_id,
        last_refresh,
        get_locations,
        search_locations,
        statistics_accounts,
        statistics_total_accounts,
        get_user_config,
        update_user_config,
        lock_deal,
        unlock_deal
    ),
    components(
        Offer,
        OfferResponse,
        Error,
        LastRefreshInformation,
        RestaurantInformation,
        AccountResponse,
        UserOptions,
        RestaurantAddress,
        AccountPointResponse,
        AccountPointMap
    )
)]
pub struct ApiDoc;

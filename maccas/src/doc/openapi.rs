use crate::routes::admin::get_locked_deals::*;
use crate::routes::admin::get_spending::*;
use crate::routes::admin::lock_deal::*;
use crate::routes::admin::register::*;
use crate::routes::admin::unlock_deal::*;
use crate::routes::auth::login::*;
use crate::routes::auth::register::*;
use crate::routes::auth::token::*;
use crate::routes::code::get_code::*;
use crate::routes::deals::add_deal::*;
use crate::routes::deals::get_deal::*;
use crate::routes::deals::get_deals::*;
use crate::routes::deals::get_last_refresh::*;
use crate::routes::deals::remove_deal::*;
use crate::routes::docs::openapi::*;
use crate::routes::health::status::*;
use crate::routes::locations::get_locations::*;
use crate::routes::locations::search_locations::*;
use crate::routes::points::get_by_id::*;
use crate::routes::points::get_points::*;
use crate::routes::statistics::get_account::*;
use crate::routes::statistics::get_total_accounts::*;
use crate::routes::user::config::*;
use crate::routes::user::spending::*;
use crate::types::api::{
    AccountPointMap, AccountPointResponse, AccountResponse, AdminLockedDealsResponse,
    AdminUserSpending, AdminUserSpendingMap, GetDealsOffer, LastRefreshInformation, LoginRequest,
    OfferPointsResponse, OfferResponse, PointsResponse, RegistrationRequest, RestaurantAddress,
    RestaurantInformation, RestaurantInformationList, TokenRequest, TokenResponse, UserSpending,
};
use crate::types::user::UserOptions;
use foundation::types::role::UserRole;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    security(("JWT" = []),("API Key" = [])),
    paths(
        get_code,
        get_deals,
        add_deal,
        remove_deal,
        get_deal,
        get_locations,
        get_last_refresh,
        search_locations,
        get_points_by_id,
        get_points,
        get_accounts,
        get_total_accounts,
        get_user_config,
        update_user_config,
        get_locked_deals,
        unlock_deal,
        lock_deal,
        get_user_spending,
        get_all_user_spending,
        get_status,
        get_openapi,
        login,
        get_token,
        register,
        registration_token
    ),
    components(
        responses(),
        schemas(
            OfferResponse,
            LastRefreshInformation,
            RestaurantInformation,
            AccountResponse,
            OfferPointsResponse,
            PointsResponse,
            UserOptions,
            RestaurantAddress,
            AccountPointResponse,
            AccountPointMap,
            AdminLockedDealsResponse,
            GetDealsOffer,
            UserSpending,
            AdminUserSpending,
            RestaurantInformationList,
            AdminUserSpendingMap,
            TokenResponse,
            LoginRequest,
            TokenRequest,
            UserRole,
            RegistrationRequest,
        ),
    )
)]
pub struct ApiDoc;

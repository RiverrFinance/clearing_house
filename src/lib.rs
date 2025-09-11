use crate::stable_memory::{ADMIN, HOUSE_SETTINGS};
use candid::Principal;
use ic_cdk::{export_candid, init};

use crate::admin_roles::add_market::CreateMarketParams;
use crate::house_settings::HouseDetails;
use crate::pricing_update_management::price_fetch::AssetPricingDetails;

// Import types needed for Candid generation
use add_liquidity::add_liquidity_params::AddLiquidityParams;
use close_position::close_position_params::ClosePositionParams;
use close_position::close_position_result::ClosePositionResult;
use deposit::deposit_params::DepositParams;
use market::functions::open_position_in_market::OpenPositioninMarketResult;
use market::market_details::LiquidityOperationResult;
use open_position::open_position_params::OpenPositionParams;
use query::market_details_query::QueryMarketDetailsResult;
use query::position_query::QueryPositionDetailsResult;
use remove_liquidity::remove_liquidity_params::RemoveLiquidityParams;
use withdraw::withdraw_params::WithdrawParams;

// Module declarations
pub mod add_liquidity;
pub mod admin_roles;
pub mod asset_management;
pub mod close_position;
pub mod constants;
pub mod deposit;
pub mod house_settings;
pub mod market;
pub mod math;
pub mod open_position;
pub mod position;
pub mod pricing_update_management;
pub mod query;
pub mod remove_liquidity;
pub mod stable_memory;
#[cfg(test)]
pub mod unit_tests;
pub mod user;
pub mod utils;
pub mod withdraw;
// Re-export all public functions that should be available as IC endpoints
// These are the functions that will be included in the generated Candid file

// Update functions
pub use add_liquidity::add_liquidity::add_liquidity;
pub use admin_roles::add_market::create_new_market;
pub use close_position::close_position::close_position;
pub use deposit::deposit::deposit_into_account;
pub use open_position::open_position::open_position;
pub use query::market_details_query::query_market_details;
pub use query::position_query::get_all_user_positions_in_market;
pub use remove_liquidity::remove_liquidity::remove_liquidity;
pub use withdraw::withdraw::withdraw_from_account;

// Query functions

pub use market::query_utils::get_market_details;
pub use user::balance_utils::get_user_balance;

#[init]
fn init(admin: Principal, init_details: HouseDetails) {
    ADMIN.with_borrow_mut(|reference| reference.set(admin));

    HOUSE_SETTINGS.with_borrow_mut(|reference| reference.set(init_details));
}

// Export Candid macro - this generates the Candid file automatically
export_candid!();

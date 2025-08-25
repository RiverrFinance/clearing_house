use candid::Principal;
use ic_cdk::{api::msg_caller, export_candid, init};

// Import types needed for Candid generation
use add_liquidity::add_liquidity_params::AddLiquidityToMarketParams;
use close_position::close_position_params::ClosePositionParams;
use close_position::close_position_result::ClosePositionResult;
use deposit::deposit_params::DepositParams;
use market::functions::open_position_in_market::OpenPositioninMarketResult;
use market::market_details::LiquidityOperationResult;
use market::market_details::MarketDetails;
use open_position::open_position_params::OpenPositionParams;
use remove_liquidity::remove_liquidity_params::RemoveLiquidityFromMarketParams;
use withdraw::withdraw_params::WithdrawParams;

// Module declarations
pub mod add_liquidity;
pub mod admin;
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
pub use admin::admin_functions::add_market;
pub use close_position::close_position::close_position;
pub use deposit::deposit::deposit;
pub use open_position::open_position::open_position;
pub use remove_liquidity::remove_liquidity::remove_liquidity;
pub use withdraw::withdraw::withdraw;

// Query functions

pub use market::query_utils::{get_market_details, get_markets_count};
pub use user::balance_utils::get_user_balance;

use crate::{
    house_settings::HouseDetails,
    stable_memory::{ADMIN, HOUSE_SETTINGS}, //, XRC},
};

#[init]
fn init(init_details: HouseDetails) {
    let admin = msg_caller();
    ADMIN.with_borrow_mut(|reference| reference.set(admin));
    //  XRC.with_borrow_mut(|reference| reference.set(xrc_id));

    HOUSE_SETTINGS.with_borrow_mut(|reference| reference.set(init_details));
}

// Export Candid macro - this generates the Candid file automatically
export_candid!();

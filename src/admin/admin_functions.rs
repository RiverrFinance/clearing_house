use candid::{CandidType, Principal};
use ic_cdk::update;
use serde::Deserialize;

use crate::market::components::bias::Bias;
use crate::market::components::funding_manager::FundingManager;
use crate::market::components::liquidity_manager::HouseLiquidityManager;
use crate::market::market_details::MarketDetails;
use crate::market::market_details::MarketState;

use crate::pricing_update_management::price_fetch::AssetPricingDetails;

use crate::stable_memory::{ADMIN, MARKETS_WITH_LAST_PRICE_UPDATE_TIME};

fn admin_guard() -> Result<(), String> {
    ADMIN.with_borrow(|admin_ref| {
        let admin = admin_ref.get().clone();
        if ic_cdk::api::msg_caller() == admin {
            return Ok(());
        } else {
            return Err("Invalid".to_string());
        };
    })
}

// #[test]
// fn test_create_market_params() {
//     let funding_factor = (0.00000002 * (FLOAT_PRECISION as f64)) as u128;
//     let funding_exponent = FLOAT_PRECISION;
//     let base_borrowing_factor_long = (0.00000000625 * (FLOAT_PRECISION as f64)) as u128;
//     let base_borrowing_factor_short = (0.00000000625 * (FLOAT_PRECISION as f64)) as u128;

//     let borrowing_exponent_long = FLOAT_PRECISION;
//     let borrowing_exponent_short = FLOAT_PRECISION;
//     let longs_reserve_factor = (0.4 * (FLOAT_PRECISION as f64)) as u128;
//     let shorts_max_reserve_factor = (0.4 * (FLOAT_PRECISION as f64)) as u128;

//     let longs_max_reserve_factor = (0.00000000625 * (FLOAT_PRECISION as f64)) as u128;

//     println!("longs_reserve_factor: {}", longs_reserve_factor);
//     println!("shorts_max_reserve_factor: {}", shorts_max_reserve_factor);
//     println!("longs_max_reserve_factor: {}", longs_max_reserve_factor);
//     println!("funding_factor: {}", funding_factor);
//     println!("base_borrowing_factor_long: {}", base_borrowing_factor_long);
//     println!(
//         "base_borrowing_factor_short: {}",
//         base_borrowing_factor_short
//     );
//     println!("funding_exponent: {}", funding_exponent);
//     println!("borrowing_exponent_long: {}", borrowing_exponent_long);
//     println!("borrowing_exponent_short: {}", borrowing_exponent_short);
//}

#[derive(Default, Deserialize, CandidType)]

pub struct CreateMarketParams {
    pub asset_pricing_details: AssetPricingDetails,
    pub init_state: MarketState,
    pub funding_factor: u128,
    pub funding_exponent_factor: u128,
    pub longs_max_reserve_factor: u128,
    pub longs_borrowing_exponent_factor: u128,
    pub longs_base_borrowing_factor: u128,
    pub shorts_max_reserve_factor: u128,
    pub shorts_borrowing_exponent_factor: u128,
    pub shorts_base_borrowing_factor: u128,
}

// #[update(name = "addMarket", guard = "admin_guard")]
#[update(name = "addMarket", guard = "admin_guard")]
pub fn add_market(
    params: CreateMarketParams,
    asset_pricing_details: AssetPricingDetails,
    init_state: MarketState,
) {
    let mut liquidity_manager = HouseLiquidityManager::default();
    liquidity_manager.longs_max_reserve_factor = params.longs_max_reserve_factor;
    liquidity_manager.shorts_max_reserve_factor = params.shorts_max_reserve_factor;

    let mut funding_manager = FundingManager::default();
    funding_manager.funding_factor = params.funding_factor;
    funding_manager.funding_exponent_factor = params.funding_exponent_factor;

    let mut bias = Bias::default();
    bias.longs.borrowing_exponent_factor_ = params.longs_borrowing_exponent_factor;
    bias.longs.base_borrowing_factor = params.longs_base_borrowing_factor;
    bias.shorts.borrowing_exponent_factor_ = params.shorts_borrowing_exponent_factor;
    bias.shorts.base_borrowing_factor = params.shorts_base_borrowing_factor;
    // bias.longs_reserve_factor = params.longs_reserve_factor;
    // bias.shorts_max_reserve_factor = params.shorts_max_reserve_factor;

    let mut market_details = MarketDetails::default();
    market_details.index_asset_pricing_details = asset_pricing_details;
    market_details.state = init_state;
    market_details.funding_manager = funding_manager;
    market_details.bias_tracker = bias;
    market_details.liquidity_manager = liquidity_manager;

    MARKETS_WITH_LAST_PRICE_UPDATE_TIME.with_borrow_mut(|reference| {
        reference.push(&(market_details, 0));
    })
}

#[update(name = "setAdmin", guard = "admin_guard")]
pub fn set_admin(new_admin: Principal) {
    ADMIN.with_borrow_mut(|reference| reference.set(new_admin));
}

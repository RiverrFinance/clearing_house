use candid::CandidType;
use ic_cdk::update;
use serde::Deserialize;

use crate::admin_roles::admin_guard;

use crate::{
    market::{
        components::{
            bias::Bias, funding_state::FundingState, liquidity_state::HouseLiquidityState,
        },
        market_details::{MarketDetails, MarketState},
    },
    pricing_update_management::price_fetch::AssetPricingDetails,
    stable_memory::MARKETS_WITH_LAST_PRICE_UPDATE_TIME,
};

#[derive(Default, Deserialize, CandidType)]

pub struct CreateMarketParams {
    #[serde(rename = "assetPricingDetails")]
    pub asset_pricing_details: AssetPricingDetails,
    #[serde(rename = "initState")]
    pub init_state: MarketState,
    #[serde(rename = "fundingFactor")]
    pub funding_factor: u128,
    #[serde(rename = "fundingExponentFactor")]
    pub funding_exponent_factor: u128,
    #[serde(rename = "longsMaxReserveFactor")]
    pub longs_max_reserve_factor: u128,
    #[serde(rename = "longsBorrowingExponentFactor")]
    pub longs_borrowing_exponent_factor: u128,
    #[serde(rename = "longsBaseBorrowingFactor")]
    pub longs_base_borrowing_factor: u128,
    #[serde(rename = "shortsMaxReserveFactor")]
    pub shorts_max_reserve_factor: u128,
    #[serde(rename = "shortsBorrowingExponentFactor")]
    pub shorts_borrowing_exponent_factor: u128,
    #[serde(rename = "shortsBaseBorrowingFactor")]
    pub shorts_base_borrowing_factor: u128,
}

// #[update(name = "addMarket", guard = "admin_guard")]
#[update(name = "createNewMarket", guard = "admin_guard")]
pub fn create_new_market(
    params: CreateMarketParams,
    asset_pricing_details: AssetPricingDetails,
) -> u64 {
    let mut liquidity_state = HouseLiquidityState::default();
    liquidity_state.longs_max_reserve_factor = params.longs_max_reserve_factor;
    liquidity_state.shorts_max_reserve_factor = params.shorts_max_reserve_factor;

    let mut funding_state = FundingState::default();
    funding_state.funding_factor = params.funding_factor;
    funding_state.funding_exponent_factor = params.funding_exponent_factor;

    let mut bias = Bias::default();
    bias.longs.borrowing_exponent_factor_ = params.longs_borrowing_exponent_factor;
    bias.longs.base_borrowing_factor = params.longs_base_borrowing_factor;
    bias.shorts.borrowing_exponent_factor_ = params.shorts_borrowing_exponent_factor;
    bias.shorts.base_borrowing_factor = params.shorts_base_borrowing_factor;
    // bias.longs_reserve_factor = params.longs_reserve_factor;
    // bias.shorts_max_reserve_factor = params.shorts_max_reserve_factor;

    let mut market_details = MarketDetails::default();
    market_details.index_asset_pricing_details = asset_pricing_details;
    market_details.state = params.init_state;
    market_details.funding_state = funding_state;
    market_details.bias_tracker = bias;
    market_details.liquidity_state = liquidity_state;

    MARKETS_WITH_LAST_PRICE_UPDATE_TIME.with_borrow_mut(|reference| {
        reference.push(&(market_details, 0));
        return reference.len() - 1;
    })
}

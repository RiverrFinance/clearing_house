use candid::CandidType;
use ic_cdk::query;
use serde::Deserialize;

use crate::{
    market::components::liquidity_state::HouseLiquidityState,
    math::math::{Neg, apply_precision, mul_div},
    stable_memory::MARKETS_LIST,
};

#[derive(CandidType, Deserialize)]
pub struct QueryMarketDetailsResult {
    #[serde(rename = "longsTotalOpenInterest")]
    longs_total_open_interest: u128,
    #[serde(rename = "shortsTotalOpenInterest")]
    shorts_total_open_interest: u128,
    #[serde(rename = "longsReserveAvailableLiquidity")]
    longs_reserve_available_liquidity: u128,
    #[serde(rename = "shortsReserveAvailableLiquidity")]
    shorts_reserve_available_liquidity: u128,
    #[serde(rename = "currentFundingFactorPerHourLong")]
    current_funding_factor_per_hour_long: i128,
    #[serde(rename = "currentFundingFactorPerHourShort")]
    current_funding_factor_per_hour_short: i128,
}

#[query(name = "queryMarketDetails")]
pub fn query_market_details(market_index: u64) -> QueryMarketDetailsResult {
    MARKETS_LIST.with_borrow(|reference| {
        let market = reference.get(market_index).unwrap();

        let shorts_total_open_interest = market.bias_tracker.total_open_interest_for_bias(false);
        let longs_total_open_interest = market.bias_tracker.total_open_interest_for_bias(true);

        let house_value_without_pnl = market.liquidity_state.static_value().max(0) as u128;

        let HouseLiquidityState {
            free_liquidity,
            current_longs_reserve,
            current_shorts_reserve,
            shorts_max_reserve_factor,
            longs_max_reserve_factor,
            ..
        } = market.liquidity_state;

        let (longs_max_reserve, current_reserve_for_longs) = (
            apply_precision(longs_max_reserve_factor, house_value_without_pnl),
            current_longs_reserve,
        );

        let (shorts_max_reserve, current_reserve_for_shorts) = (
            apply_precision(shorts_max_reserve_factor, house_value_without_pnl),
            current_shorts_reserve,
        );

        let longs_reserve_available_liquidity =
            (shorts_max_reserve - current_reserve_for_shorts).min(free_liquidity);
        let shorts_reserve_available_liquidity =
            (longs_max_reserve - current_reserve_for_longs).min(free_liquidity);

        let current_funding_factor_per_sec = market.funding_state.current_funding_factor_ps();

        let current_funding_factor_per_hour_long;
        let current_funding_factor_per_hour_short;

        if longs_total_open_interest == 0 || shorts_total_open_interest == 0 {
            current_funding_factor_per_hour_long = 0;
            current_funding_factor_per_hour_short = 0;
        } else {
            if current_funding_factor_per_sec > 0 {
                // longs pay shorts
                current_funding_factor_per_hour_long = current_funding_factor_per_sec.neg() * 3600; //ONE HOUR
                current_funding_factor_per_hour_short = mul_div(
                    current_funding_factor_per_hour_long.abs() as u128,
                    longs_total_open_interest,
                    shorts_total_open_interest,
                ) as i128;
            } else {
                current_funding_factor_per_hour_short = current_funding_factor_per_sec * 3600;
                current_funding_factor_per_hour_long = mul_div(
                    current_funding_factor_per_hour_short.abs() as u128,
                    shorts_total_open_interest,
                    longs_total_open_interest,
                ) as i128;
            }
        }

        QueryMarketDetailsResult {
            longs_total_open_interest,
            shorts_total_open_interest,
            longs_reserve_available_liquidity,
            shorts_reserve_available_liquidity,
            current_funding_factor_per_hour_long,
            current_funding_factor_per_hour_short,
        }
    })
}

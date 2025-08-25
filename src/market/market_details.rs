use std::{borrow::Cow, cmp::max};

use candid::CandidType;
use serde::{Deserialize, Serialize};

use ic_stable_structures::storable::{Bound, Storable};

use crate::market::components::bias::Bias;
use crate::market::components::funding_manager::FundingManager;
use crate::market::components::liquidity_manager::HouseLiquidityManager;
use crate::market::components::pricing::PricingManager;
use crate::math::math::to_precision;
use crate::pricing_update_management::price_fetch::AssetPricingDetails;

#[derive(CandidType, Deserialize)]
pub enum LiquidityOperationResult {
    Settled { amount_out: u128 },
    Waiting,
    Failed,
}

#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
#[derive(Default, Deserialize, Serialize, CandidType, Clone, Copy)]
pub struct MarketState {
    pub max_leverage_factor: u128,
    pub max_reserve_factor: u128,
    pub liquidation_factor: u128,
}

#[cfg_attr(test, derive(Debug, Clone, PartialEq, Eq))]
#[derive(Default, Deserialize, Serialize, CandidType)]
pub struct MarketDetails {
    pub index_asset_pricing_details: AssetPricingDetails,
    pub token_identifier: String,
    pub bias_tracker: Bias,
    pub funding_manager: FundingManager,
    pub pricing_manager: PricingManager,
    pub state: MarketState,
    pub liquidity_manager: HouseLiquidityManager,
}

impl MarketDetails {
    pub fn _update_price(&mut self, price: u64, decimal: u32) -> u128 {
        let Self {
            pricing_manager, ..
        } = self;

        let price_to_precision = to_precision(price as u128, 10u128.pow(decimal));
        pricing_manager.update_price(price_to_precision);
        price_to_precision
    }

    /// Calculates the current value of the
    pub fn _house_value(&mut self, price: u128) -> u128 {
        let house_value = max(
            self.liquidity_manager.static_value() as i128 - self.bias_tracker.net_house_pnl(price),
            0,
        );

        return house_value as u128;
    }
    pub fn index_asset_pricing_details(&self) -> AssetPricingDetails {
        self.index_asset_pricing_details.clone()
    }

    pub fn get_cummulative_funding_factor_since_epoch(&self, is_long_position: bool) -> i128 {
        let Self { bias_tracker, .. } = self;
        let Bias { longs, shorts, .. } = bias_tracker;
        if is_long_position {
            longs.cummulative_funding_factor_since_epoch()
        } else {
            shorts.cummulative_funding_factor_since_epoch()
        }
    }

    pub fn get_cummulative_borrowing_factor_since_epoch(&self, is_long_position: bool) -> u128 {
        let Self { bias_tracker, .. } = self;
        let Bias { longs, shorts, .. } = bias_tracker;
        if is_long_position {
            longs.cummulative_borrowing_factor_since_epcoh()
        } else {
            shorts.cummulative_borrowing_factor_since_epcoh()
        }
    }

    // Calculates the Price imapct for opening a position
    //
    // checks if position opening is chnages the currentr skew direction i.e (longs greater than shorts )
    // then calls pricing manager to get price impact fee ,if pricing impact
    // fn calculate_price_impact_open_position(&self, open_interest: u128) -> i128 {
    //     let Self {
    //         bias_tracker,
    //         pricing_manager,
    //         ..
    //     } = self;
    //     let current_net_long_open_interest = bias_tracker.long.traders_open_interest();
    //     let current_net_short_open_interest = bias_tracker.short.traders_open_interest();

    //     let initial_diff = bias_tracker.long_short_open_interest_diff();

    //     let next_diff = (current_net_long_open_interest + open_interest) as i128
    //         - current_net_short_open_interest as i128;

    //     let same_side_rebalance =
    //         (initial_diff > 0 && next_diff > 0) || (initial_diff < 0 && next_diff < 0);
    //     let price_impact_fee;
    //     if same_side_rebalance {
    //         price_impact_fee = pricing_manager.get_price_impact_for_same_side_rebalance(
    //             initial_diff.abs() as u128,
    //             next_diff.abs() as u128,
    //         );
    //     } else {
    //         price_impact_fee = pricing_manager.get_price_impact_for_crossover_rebalance(
    //             initial_diff.abs() as u128,
    //             next_diff.abs() as u128,
    //         );
    //     }
    //     return price_impact_fee;
    // }

    //price impact = (initial USD difference) ^ (price impact exponent) * (price impact factor) - (next USD difference) ^ (price impact exponent) * (price impact factor)
}

impl Storable for MarketDetails {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let serialized = bincode::serialize(self).expect("failed to serialize");
        Cow::Owned(serialized)
    }

    /// Converts the element into an owned byte vector.
    ///
    /// This method consumes `self` and avoids cloning when possible.
    fn into_bytes(self) -> Vec<u8> {
        bincode::serialize(&self).expect("failed to serialize")
    }

    /// Converts bytes into an element.
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        bincode::deserialize(bytes.as_ref()).expect("failed to desearalize")
    }

    /// The size bounds of the type.
    const BOUND: Bound = Bound::Unbounded;
}

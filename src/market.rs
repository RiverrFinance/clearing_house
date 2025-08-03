use bincode::{self, Decode, Encode};

use std::{borrow::Cow, collections};

use ic_stable_structures::storable::{Bound, Storable};

use crate::{
    bias::Bias,
    funding::FundingManager,
    math::{diff, mul_div, to_precision},
    pricing::PricingManager,
    types::Asset,
};

pub const MAX_ALLOWED_PRICE_CHANGE_INTERVAL: u64 = 600_000_000_000;

#[derive(Encode, Decode, Default)]
pub struct MarketState {
    pub max_leverage_x10: u8,
    pub max_pnl: u64,
    pub min_collateral: u128,
    pub execution_fee: u128,
}

#[derive(Encode, Decode, Default)]
pub struct MarketDetails {
    pub base_asset: Asset,
    pub token_identifier: u128,
    pub bias_tracker: Bias,
    pub funding_manager: FundingManager,
    pub pricing_manager: PricingManager,
    pub state: MarketState,
    pub total_deposit: u128,
    pub max_position_reserve: u128,
    pub positions_reserve: u128,
}

impl MarketDetails {
    fn calculate_price_impact_open_position(&self, open_interest: u128) -> i128 {
        let Self {
            bias_tracker,
            pricing_manager,
            ..
        } = self;
        let current_net_long_open_interest = bias_tracker.long.traders_open_interest();
        let current_net_short_open_interest = bias_tracker.short.traders_open_interest();

        let initial_diff = bias_tracker.long_short_open_interest_diff().abs() as u128;

        let next_diff = (current_net_long_open_interest + open_interest) as i128
            - current_net_short_open_interest as i128;

        let same_side_rebalance =
            (initial_diff > 0 && next_diff > 0) || (initial_diff < 0 && next_diff < 0);
        let price_impact_fee;
        if same_side_rebalance {
            price_impact_fee = pricing_manager
                .get_price_impact_for_same_side_rebalance(initial_diff, next_diff.abs() as u128);
        } else {
            price_impact_fee = pricing_manager
                .get_price_impact_for_crossover_rebalance(initial_diff, next_diff.abs() as u128);
        }
        return price_impact_fee;
    }

    fn calculate_reserve_in_for_opening_position(
        &mut self,
        price: u128,
        long: bool,
        max_profit: u128,
    ) -> u128 {
        let mut added_reserve = max_profit;
        let Self { bias_tracker, .. } = self;
        let current_unrealized_reserve_for_long =
            bias_tracker.current_unrealised_reserve_for_longs(price);

        let current_unrealized_reserve_for_short =
            bias_tracker.current_unrealised_reserve_for_shorts(price);

        let condition = current_unrealized_reserve_for_short > current_unrealized_reserve_for_long;

        let reserve_in_excess = condition && long || !condition && !long;

        if reserve_in_excess {
            let diff_in_unrealised_reserve = diff(
                current_unrealized_reserve_for_short,
                current_unrealized_reserve_for_long,
            );

            // checks if max _profit exceeds remaining reserve
            if max_profit > diff_in_unrealised_reserve {
                added_reserve = max_profit - diff_in_unrealised_reserve
            } else {
                added_reserve = 0
            }
        }

        return added_reserve;
    }

    ////
    ///
    ///
    ///
    ///
    ///
    ///
    fn _open_psoition(&mut self, collateral: u128, debt: u128, max_profit: u128, long: bool) {
        let free_liuqidity = self.free_liquidity();
        // fail conditions

        let Self {
            pricing_manager, ..
        } = self;

        let price_update =
            pricing_manager.get_price_within_interval(MAX_ALLOWED_PRICE_CHANGE_INTERVAL);
        // refactor later
        let price = price_update.unwrap_or(0);

        let added_reserve = self.calculate_reserve_in_for_opening_position(price, long, max_profit);

        if debt + added_reserve >= free_liuqidity {};
        if debt + added_reserve + self.positions_reserve > self.max_position_reserve {};

        let position_open_interest = debt + collateral;

        let units = to_precision(max_profit, price);

        self.update_bias_details(
            position_open_interest as i128,
            max_profit as i128,
            units as i128,
            long,
        );

        // get the amount
    }

    fn update_bias_details(
        &mut self,
        delta_toi: i128,
        delta_ra: i128,
        delta_hpu: i128,
        is_long_position: bool,
    ) {
        let Self { bias_tracker, .. } = self;

        bias_tracker.update_bias_details(delta_toi, delta_ra, delta_hpu, is_long_position);
    }

    fn free_liquidity(&self) -> u128 {
        self.total_deposit - self.positions_reserve
    }
    /// Update funding
    ///
    /// functionis called on interval for payment of funding fees
    /// funding fee to paid after a duration is caluculated
    pub fn update_funding(&mut self) {
        let Self {
            funding_manager,
            bias_tracker,
            ..
        } = self;

        let Bias { long, short } = bias_tracker;

        let current_funding_factor_ps = funding_manager.current_funding_factor_ps();

        let duration = funding_manager._seconds_since_last_update();

        let majority_funding_fee = current_funding_factor_ps * duration as i128;

        let long_open_interest = long.traders_open_interest();

        let short_open_interest = short.traders_open_interest();

        if current_funding_factor_ps <= 0 {
            // shorts pay long

            short.update_cumulative_funding_factor(majority_funding_fee);

            // long fee = (majority funding * short open interest)/ long_open_interest
            let long_funding_fee = mul_div(
                majority_funding_fee.abs() as u128,
                short_open_interest,
                long_open_interest,
            ) * duration;
            long.update_cumulative_funding_factor(long_funding_fee as i128);
        } else {
            //longs pay short
            long.update_cumulative_funding_factor(majority_funding_fee * -1);

            // short fee  = (majority_funding_fee * long open interest) / short_open_interest
            let short_funding_fee = mul_div(
                majority_funding_fee.abs() as u128,
                long_open_interest,
                short_open_interest,
            ) * duration;
            short.update_cumulative_funding_factor(short_funding_fee as i128)
        }

        let long_short_diff = long_open_interest as i128 - short_open_interest as i128;

        let total_open_interest = long_open_interest + short_open_interest;

        funding_manager._update_funding_factor_ps(long_short_diff, total_open_interest);
    }
    pub fn open_positon_at_current_price() {}
}
//price impact = (initial USD difference) ^ (price impact exponent) * (price impact factor) - (next USD difference) ^ (price impact exponent) * (price impact factor)

impl Storable for MarketDetails {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let mut slice = [0u8; 500];
        let length =
            bincode::encode_into_slice(self, &mut slice, bincode::config::standard()).unwrap();

        let slice = &slice[..length];
        Cow::Owned(slice.to_vec())
    }

    /// Converts the element into an owned byte vector.
    ///
    /// This method consumes `self` and avoids cloning when possible.
    fn into_bytes(self) -> Vec<u8> {
        let mut slice = [0u8; 500];
        let length =
            bincode::encode_into_slice(self, &mut slice, bincode::config::standard()).unwrap();

        let slice = &slice[..length];
        slice.to_vec()
    }

    /// Converts bytes into an element.
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        bincode::decode_from_slice(bytes.as_ref(), bincode::config::standard())
            .expect("Failed to decode MarketDetails")
            .0
    }

    /// The size bounds of the type.
    const BOUND: Bound = Bound::Unbounded;

    /// Like `to_bytes`, but checks that bytes conform to declared bounds.
    fn to_bytes_checked(&self) -> Cow<'_, [u8]> {
        let bytes = self.to_bytes();
        Self::check_bounds(&bytes);
        bytes
    }

    /// Like `into_bytes`, but checks that bytes conform to declared bounds.
    fn into_bytes_checked(self) -> Vec<u8>
    where
        Self: Sized,
    {
        let bytes = self.into_bytes();
        Self::check_bounds(&bytes);
        bytes
    }

    #[inline]
    fn check_bounds(bytes: &[u8]) {
        if let Bound::Bounded {
            max_size,
            is_fixed_size,
        } = Self::BOUND
        {
            let actual = bytes.len();
            if is_fixed_size {
                assert_eq!(
                    actual, max_size as usize,
                    "expected a fixed-size element with length {} bytes, but found {} bytes",
                    max_size, actual
                );
            } else {
                assert!(
                    actual <= max_size as usize,
                    "expected an element with length <= {} bytes, but found {} bytes",
                    max_size,
                    actual
                );
            }
        }
    }
}

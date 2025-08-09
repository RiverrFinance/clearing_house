use bincode::{self};
use candid::Principal;
use ic_cdk::api::time;
use serde::{Deserialize, Serialize};

use std::borrow::Cow;

use ic_stable_structures::storable::{Bound, Storable};

use crate::{
    bias::{Bias, UpdateBiasDetailsParamters},
    funding::FundingManager,
    math::{apply_precision, bound_above_signed, bound_below_signed, mul_div, to_precision},
    pricing::PricingManager,
    types::Asset,
};

pub const MAX_ALLOWED_PRICE_CHANGE_INTERVAL: u64 = 600_000_000_000;

pub enum OpenPositionResult {
    // Limit {acceptable_price:u128,position:Position},
    Settled {
        position: Position,
    },
    Waiting {
        acceptable_price: u128,
        position: Position,
    },
    Failed,
}

pub enum ClosePositionResult {
    Settled {
        returns: u128,
    },
    Waiting {
        acceptable_price: u128,
        position: Position,
    },
}

#[derive(Copy, Clone)]
pub struct Position {
    pub owner: Principal,
    pub collateral: u128,
    pub long: bool,
    pub debt: u128,
    pub units: u128,
    pub max_reserve: u128,
    // the max reserve equiavlent in base token i.e amount BTC/USD market with reserve as usd then
    // max unit is amount of btc equiavlent to max reserve
    pub time_stamp: u64,

    /// cummulttive funding factor at time  of opening or updating position
    pub pre_cummulative_funding_factor: i128,
    /// cummulttive funding factor at point of opening or updating position
    pub pre_cummulative_borrowing_factor: u128,
}

impl Position {
    pub fn get_pnl(&self, price: u128) -> i128 {
        let Self {
            long,
            units,
            collateral,
            debt,
            max_reserve,
            ..
        } = *self;

        let units_value = apply_precision(price, units) as i128;

        let pnl = units_value - (collateral + debt) as i128;
        let sign = if long { 1 } else { -1 };
        bound_above_signed(pnl * sign, max_reserve as i128)
    }

    /// Returns net free liquidity,funding_to_be_paid,collateral_out,house_debt
    ///
    ///
    pub fn close_position_with_net_positive_funding(
        &self,
        free_liquidity: u128,
        net_funding_fee_magnitude: i128,
        net_borrowing_fee: u128,
        position_pnl: i128,
    ) -> (u128, u128, u128) {
        let position_pnl_magnitude = position_pnl.abs();
        let open_interest = self.open_interest();
        // let net_borrowing_fee = self.get_net_borrowing_fee(net_borrowing_fee);
        // let net_funding_fee_magnitude = self.get_net_funding_fee(net_funding_fee);
        let mut net_free_liquidity = free_liquidity;

        let net_position_value = bound_below_signed(
            open_interest as i128 + position_pnl + net_funding_fee_magnitude
                - (net_borrowing_fee + self.debt) as i128,
            0,
        );

        if position_pnl.is_negative() {
            let position_value_without_pnl = bound_below_signed(
                open_interest as i128 - (net_borrowing_fee + self.debt) as i128
                    + net_funding_fee_magnitude,
                0,
            );

            let position_loss = position_value_without_pnl.min(position_pnl_magnitude);
            //
            //
            net_free_liquidity += self.max_reserve + position_loss as u128;
        } else {
            net_free_liquidity += (self.max_reserve as i128 - position_pnl_magnitude) as u128
        }

        return (net_free_liquidity, net_position_value as u128, 0);

        //net_free_liquidity += reserve
    }

    /// Returns  (net_free_liquidity, funding_paid, collateral_out,bad_debt)
    pub fn close_position_with_net_negative_funding(
        &self,
        free_liquidity: u128,
        net_funding_fee: i128,
        net_borrowing_fee: u128,
        position_pnl: i128,
    ) -> (u128, u128, u128) {
        let position_pnl_magnitude = position_pnl.abs();
        let open_interest = self.open_interest();

        let net_funding_fee_magnitude = net_funding_fee.abs();

        let mut net_free_liquidity = free_liquidity;

        // if pnl is negative
        // the amount to add to free_liquidity is

        let position_value_without_funding_pay = bound_below_signed(
            open_interest as i128 + position_pnl - (net_borrowing_fee + self.debt) as i128,
            0,
        );

        // let mut funding_paid = net_funding_fee_magnitude as u128;

        let mut collateral_out = 0;

        let mut house_debt = 0;

        if position_value_without_funding_pay > net_funding_fee_magnitude {
            // if position_value after debt is enough to pay funding fee
            // collateral is what is left
            collateral_out =
                (position_value_without_funding_pay - net_funding_fee_magnitude) as u128;

            // net free liquidity is max_reserve - position_pnl
            net_free_liquidity += (self.max_reserve as i128 - position_pnl) as u128
        } else {
            // This block of code is tracks extreme cases like
            // when position funding_fee_can not be paid
            if position_pnl.is_negative() {
                let position_value_without_pnl = bound_below_signed(
                    open_interest as i128
                        - (net_borrowing_fee + self.debt) as i128
                        - net_funding_fee_magnitude,
                    0,
                );

                // amount extractabel to house
                let position_loss = position_value_without_pnl.min(position_pnl_magnitude);

                //
                //
                net_free_liquidity += self.max_reserve + position_loss as u128;
            } else {
                net_free_liquidity += (self.max_reserve as i128 - position_pnl_magnitude) as u128
            }

            let delta = i128::max(
                net_free_liquidity as i128 + position_value_without_funding_pay
                    - net_funding_fee_magnitude,
                0,
            );

            if delta >= 0 {
                net_free_liquidity = delta as u128;
            } else {
                net_free_liquidity = 0;
                house_debt = delta.abs() as u128;
                //funding_paid = (delta + net_funding_fee_magnitude) as u128
            }
        }

        return (net_free_liquidity, collateral_out, house_debt);
    }

    pub fn get_net_borrowing_fee(&self, net_borrowing_factor: u128) -> u128 {
        let open_interest = self.open_interest();
        return apply_precision(net_borrowing_factor, open_interest);
    }

    pub fn get_net_funding_fee(&self, net_funding_factor: i128) -> i128 {
        // if net_funding is positive return
        let sign = if net_funding_factor > 0 { 1 } else { -1 };

        let open_interest = self.open_interest();

        return apply_precision(net_funding_factor.abs() as u128, open_interest) as i128 * sign;
    }

    pub fn open_interest(&self) -> u128 {
        self.debt + self.collateral
    }
}

impl Default for Position {
    fn default() -> Self {
        Self {
            owner: Principal::anonymous(),
            long: false,
            collateral: 0,
            max_reserve: 0,
            debt: 0,
            units: 0,
            time_stamp: 0,
            pre_cummulative_borrowing_factor: 0,
            pre_cummulative_funding_factor: 0,
        }
    }
}

#[derive(Default, Deserialize, Serialize)]
pub struct MarketState {
    pub max_leverage_x10: u8,
    pub max_pnl_percentage: u64,
    pub min_collateral: u128,
    pub execution_fee: u128,
    pub liquidation_factor: u128,
}

#[derive(Default, Deserialize, Serialize)]
pub struct MarketDetails {
    pub base_asset: Asset,
    pub token_identifier: String,
    pub bias_tracker: Bias,
    pub funding_manager: FundingManager,
    pub pricing_manager: PricingManager,
    pub state: MarketState,
    pub total_deposit: u128,
    pub free_liquidity: u128,
    pub max_reserve_factor_longs: u128,
    pub max_reserve_factor_shorts: u128,
    pub current_net_positions_reserve: u128,
    pub current_net_debt: u128,
    pub bad_debt: u128,
}

impl MarketDetails {
    pub fn _house_value(&mut self, price: u128) -> u128 {
        // The House value is next sum of tokens in the pool
        // i.e
        ((self.free_liquidity + self.current_net_positions_reserve + self.current_net_debt
            - self.bad_debt) as i128
            + self.bias_tracker.house_pnl(price)) as u128
    }
    ///
    ///
    /// To Close Position
    /// close position
    /// gett the pnl from
    /// if long
    /// units value - oepn interest bound above by reserve
    /// if short
    /// open_interest - units value  bound above by reserve
    /// gets the net sum of funding factors by current_cumulative _funding factor - cummulative_funding_factor
    /// gets the sum of borrowing factors by current_cummulative_funding_factor - cummulative_funding_factor  
    /// get net funding fee
    /// position value = (open_interest + pnl - borrowing fee ) is bound below debt
    ///
    /// position_value_after_debt = position_value - debt_value
    ///if postion_value_after_debt < funding_fee
    ///    let delta =free_liquidity +  position_reserve - (funding_fee - position_value_after_debt ) bouded below by zero
    ///   if delta is greater than or equal  0 :
    ///       free_liquidity = delta
    ///    if delta is less than zero :
    ///         funding_fee_paid = (delta + funding_fee).abs() as u128
    /// else
    ///     free_liquidity =free_liquidity + position_reserve
    /// cummultive_borrowing_fee_tracker -= borrowing fee
    /// reduce
    /// if position is 0
    /// if position value = 0 amount repaid is zero else if position value -debt > zero debt else fully repaid else if
    ///  if next sum funding factor is positive add to position value
    /// if negative subtract from position value
    ///  
    pub fn close_position(
        &mut self,
        position: Position,
        acceptable_price: u128,
    ) -> ClosePositionResult {
        let Position { long, .. } = position;
        let price_update = self
            .pricing_manager
            .get_price_within_interval(MAX_ALLOWED_PRICE_CHANGE_INTERVAL);

        if let Some(price) = price_update {
            let current_cummulative_funding_factor = if long {
                self.bias_tracker
                    .long
                    .cummulative_funding_factor_since_epoch()
            } else {
                self.bias_tracker
                    .short
                    .cummulative_funding_factor_since_epoch()
            };

            let current_cummulative_borrowing_factor = if long {
                self.bias_tracker
                    .long
                    .cummulative_borrowing_factor_since_epcoh()
            } else {
                self.bias_tracker
                    .short
                    .cummulative_borrowing_factor_since_epcoh()
            };

            let net_borrowing_factor =
                current_cummulative_borrowing_factor - position.pre_cummulative_borrowing_factor;
            let net_funding_factor =
                current_cummulative_funding_factor - position.pre_cummulative_funding_factor;

            let net_borrowing_fee = position.get_net_borrowing_fee(net_borrowing_factor);
            // negative
            let net_funding_fee = position.get_net_funding_fee(net_funding_factor);

            let position_pnl = position.get_pnl(price);

            let (net_free_liquidity, collateral_out, bad_debt) = if net_funding_factor < 0 {
                position.close_position_with_net_negative_funding(
                    self.free_liquidity,
                    net_funding_fee,
                    net_borrowing_fee,
                    position_pnl,
                )
            } else {
                position.close_position_with_net_positive_funding(
                    self.free_liquidity,
                    net_funding_fee,
                    net_borrowing_fee,
                    position_pnl,
                )
            };

            let position_open_interest = position.open_interest();
            let delta_open_interest_dynamic = bound_below_signed(
                position.open_interest() as i128 + net_funding_fee - net_borrowing_fee as i128,
                position.debt as i128,
            );

            self.update_bias_details(
                position_open_interest as i128 * -1,
                delta_open_interest_dynamic,
                position.units as i128 * -1,
                position.debt as i128 * -1,
                position.max_reserve as i128 * -1,
                position.long,
            );

            self.bad_debt += bad_debt;
            self.current_net_debt -= position.debt;
            self.current_net_positions_reserve -= position.max_reserve;
            self.free_liquidity = net_free_liquidity;

            ClosePositionResult::Settled {
                returns: collateral_out,
            }
        } else {
            ClosePositionResult::Waiting {
                acceptable_price,
                position,
            }
        }
    }

    ////
    ///
    ///
    ///
    ///
    ///
    ///
    pub fn open_position(
        &mut self,
        owner: Principal,
        collateral: u128,
        debt: u128,
        long: bool,
        max_pnl: u128,
        acceptable_price: u128,
    ) -> OpenPositionResult {
        let Self {
            max_reserve_factor_longs,
            max_reserve_factor_shorts,
            current_net_positions_reserve,
            free_liquidity,
            ..
        } = *self;

        // Gets price within the required time interval
        let price_update = self
            .pricing_manager
            .get_price_within_interval(MAX_ALLOWED_PRICE_CHANGE_INTERVAL);
        // refactor later
        if let Some(price) = price_update {
            let added_reserve = max_pnl;
            //self.calculate_reserve_in_for_opening_position(price, long, max_pnl);

            let max_reserve;
            if long {
                max_reserve = apply_precision(max_reserve_factor_longs, self._house_value(price));
            } else {
                max_reserve = apply_precision(max_reserve_factor_shorts, self._house_value(price));
            }

            if debt + added_reserve > free_liquidity
                || added_reserve + current_net_positions_reserve > max_reserve
            {
                return OpenPositionResult::Failed;
            }

            self.current_net_positions_reserve += added_reserve;
            let position_open_interest = debt + collateral;

            let units = to_precision(position_open_interest, price);

            self.update_bias_details(
                position_open_interest as i128,
                position_open_interest as i128,
                debt as i128,
                debt as i128,
                max_pnl as i128,
                long,
            );

            self.free_liquidity = self.free_liquidity - (added_reserve + debt);
            self.current_net_debt += debt;
            self.current_net_positions_reserve += added_reserve;
            let current_cumulative_funding_fee = self.get_cummulative_funding_fee_since_epoch(long);

            // let price_impact = self.calculate_price_impact_open_position(position_open_interest);

            let position = Position {
                owner,
                collateral,
                long,
                debt,
                max_reserve: max_pnl,
                units,
                time_stamp: time(),
                pre_cummulative_funding_factor: current_cumulative_funding_fee,
                pre_cummulative_borrowing_factor: 0,
            };
            OpenPositionResult::Settled { position }
        } else {
            // If
            let mut position = Position::default();

            position.long = long;
            position.collateral = collateral;
            position.debt = debt;

            OpenPositionResult::Waiting {
                acceptable_price,
                position,
            }
        }

        // get the amount
    }

    pub fn base_asset(&self) -> Asset {
        self.base_asset.clone()
    }
    pub fn _update_price(&mut self, price: u64, decimal: u32) {
        let Self {
            pricing_manager, ..
        } = self;

        let price_to_precision = to_precision(price as u128, 10u128.pow(decimal));
        pricing_manager.update_price(price_to_precision);
    }
    /// Calculates the Price imapct for opening a position
    ///
    /// checks if position opening is chnages the currentr skew direction i.e (longs greater than shorts )
    /// then calls pricing manager to get price impact fee ,if pricing impact  
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

    fn get_cummulative_funding_fee_since_epoch(&mut self, is_long_position: bool) -> i128 {
        let Self { bias_tracker, .. } = self;
        let Bias { long, short } = bias_tracker;
        if is_long_position {
            long.cummulative_funding_factor_since_epoch()
        } else {
            short.cummulative_funding_factor_since_epoch()
        }
    }

    fn update_bias_details(
        &mut self,
        delta_total_open_interest: i128,
        delta_net_open_interest_dynamic: i128,
        delta_total_units: i128,
        delta_net_debt_of_traders: i128,
        delta_net_reserve: i128,
        is_long_position: bool,
    ) {
        let Self { bias_tracker, .. } = self;

        let params = UpdateBiasDetailsParamters {
            delta_net_debt_of_traders,
            delta_total_open_interest_dynamic: delta_net_open_interest_dynamic,
            delta_total_open_interest,
            delta_total_units,
            delta_net_reserve,
        };

        bias_tracker.update_bias_details(params, is_long_position);
    }

    // fn free_liquidity(&self) -> u128 {
    //     self.total_deposit - self.current_net_positions_reserve
    // }
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
///
///
///
///
///
///
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

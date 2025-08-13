use bincode::{self};
use candid::Principal;
use ic_cdk::api::time;
use serde::{Deserialize, Serialize};

use std::borrow::Cow;

use ic_stable_structures::storable::{Bound, Storable};

use crate::{
    asset::AssetPricingDetails,
    bias::{Bias, UpdateBiasDetailsParamters},
    funding::FundingManager,
    math::math::{Neg, apply_precision, mul_div, to_precision},
    position::Position,
    pricing::PricingManager,
};

pub const MAX_ALLOWED_PRICE_CHANGE_INTERVAL: u64 = 600_000_000_000;

pub enum OpenPositionResult {
    // Limit {acceptable_price:u128,position:Position},
    Settled { position: Position },
    Waiting { position: Position },
    Failed,
}

pub enum ClosePositionResult {
    Settled { returns: u128 },
    Waiting { position: Position },
    Failed,
}

pub enum LiquidityOperationResult {
    Settled { amount_out: u128 },
    Waiting,
    Failed,
}

#[derive(Default, Deserialize, Serialize)]
pub struct MarketState {
    pub max_leverage_x10: u8,
    pub max_pnl_factor: u128,
    pub min_collateral: u128,
    pub liquidation_factor: u128,
}

#[derive(Default, Deserialize, Serialize)]
pub struct HouseLiquidityManager {
    pub total_liquidity_tokens_minted: u128,
    /// Total Deposit
    ///
    /// total deposit into a amrket by both liquidity providers and traders
    pub total_deposit: u128,
    /// Current Net Positions Reserve
    ///
    /// current net positions reserve
    pub current_longs_reserve: u128,
    /// Current Net Positions Reserve
    ///
    /// current net positions reserve
    pub current_shorts_reserve: u128,
    /// Current Net debt
    ///
    /// the total debt owed by both longs and shorts
    pub current_net_debt: u128,
    /// Bad Debt
    ///
    /// bad debt owed by house
    /// Bad debt occurrs when positons that shold be liquidated are not liquidated on time, the positions debt on funding fees given
    /// to the house fot the particular market ,  
    pub bad_debt: u128,
    /// Free Liquidity
    ///
    /// unsused house liquidity
    pub free_liquidity: u128,
    /// Current Borrow Fees Owed
    ///
    /// The current borrow fees collected for for all current opened positons
    /// This tracks the current borrow fees that is unpaid by current open positons
    pub current_borrow_fees_owed: u128,
    /// Longs Max Reserve Factor
    ///  
    ///
    pub longs_max_reserve_factor: u128,
    pub shorts_max_reserve_factor: u128,
    pub last_time_since_borrow_fees_collected: u64,
}

impl HouseLiquidityManager {
    /// The House value is difference of the net sum of tokens in the pool (excluding losses or gains from traders positions)
    /// and the current bad debt of the pool
    /// @dev it is returned as a signed integer becasue in rare cases of extreme bad debt ,this value might be less than zero
    pub fn static_value(&self) -> i128 {
        (self.free_liquidity
            + self.current_longs_reserve
            + self.current_shorts_reserve
            + self.current_net_debt
            + self.current_borrow_fees_owed) as i128
            - self.bad_debt as i128
    }
}

#[derive(Default, Deserialize, Serialize)]
pub struct MarketDetails {
    pub asset_pricing_details: AssetPricingDetails,
    pub token_identifier: String,
    pub bias_tracker: Bias,
    pub funding_manager: FundingManager,
    pub pricing_manager: PricingManager,
    pub state: MarketState,
    pub liquidity_manager: HouseLiquidityManager,
}

impl MarketDetails {
    pub fn deposit_liquidity(
        &mut self,
        deposit_amount: u128,
        min_out: u128,
    ) -> LiquidityOperationResult {
        let price_update = self
            .pricing_manager
            .get_price_within_interval(MAX_ALLOWED_PRICE_CHANGE_INTERVAL);

        if let Some(price) = price_update {
            let house_value = self._house_value(price);

            let Self {
                liquidity_manager, ..
            } = self;

            let HouseLiquidityManager {
                total_deposit,
                total_liquidity_tokens_minted,
                free_liquidity,
                bad_debt,
                ..
            } = liquidity_manager;

            let liquidity_tokens_to_mint = if house_value == 0 {
                deposit_amount
            } else {
                mul_div(deposit_amount, *total_liquidity_tokens_minted, house_value)
            };

            if liquidity_tokens_to_mint < min_out {
                return LiquidityOperationResult::Failed;
            }
            *total_deposit += deposit_amount;

            let repaid_bad_debt = (*bad_debt).min(deposit_amount);
            if repaid_bad_debt == *bad_debt {
                // amount is enough to repay bad debt ;

                *free_liquidity += deposit_amount - repaid_bad_debt;
            }
            *bad_debt -= repaid_bad_debt;

            *total_liquidity_tokens_minted += liquidity_tokens_to_mint;

            return LiquidityOperationResult::Settled {
                amount_out: liquidity_tokens_to_mint,
            };
        } else {
            return LiquidityOperationResult::Waiting;
        }
    }

    pub fn withdraw_liquidity(
        &mut self,
        liquidity_tokens_in: u128,
        min_out: u128,
    ) -> LiquidityOperationResult {
        let price_update = self
            .pricing_manager
            .get_price_within_interval(MAX_ALLOWED_PRICE_CHANGE_INTERVAL);

        if let Some(price) = price_update {
            let house_value = self._house_value(price);

            let Self {
                liquidity_manager, ..
            } = self;

            let HouseLiquidityManager {
                total_deposit,
                total_liquidity_tokens_minted,
                free_liquidity,
                ..
            } = liquidity_manager;

            let amount_of_assets_out = mul_div(
                liquidity_tokens_in,
                house_value,
                *total_liquidity_tokens_minted,
            );

            let amount_available = amount_of_assets_out.min(*free_liquidity);

            if amount_available < min_out {
                return LiquidityOperationResult::Failed;
            }

            *free_liquidity -= amount_available;
            *total_deposit -= amount_available;

            LiquidityOperationResult::Settled {
                amount_out: amount_available,
            }
        } else {
            return LiquidityOperationResult::Waiting;
        }
    }

    ///
    ///Open Position
    ///
    /// Parameters
    ///
    /// Owner the owner of the position
    /// Collateral : collateralfor opening position
    /// Debt - debt corresponding to leverage
    /// Long - true for long
    /// MAX PNL - the ma reserve for formation
    /// ACCEPTABLE PRICE - the price limit for  
    pub fn open_position(
        &mut self,
        owner: Principal,
        collateral: u128,
        debt: u128,
        long: bool,
        max_pnl: u128,
        acceptable_price_limit: u128,
    ) -> OpenPositionResult {
        // Gets price within the required time interval
        let price_update = self
            .pricing_manager
            .get_price_within_interval(MAX_ALLOWED_PRICE_CHANGE_INTERVAL);
        // refactor later
        if let Some(price) = price_update {
            if (long && price > acceptable_price_limit)
                || ((!long) && price < acceptable_price_limit)
            {
                return OpenPositionResult::Failed;
            }
            let added_reserve = max_pnl;

            let house_value = self._house_value(price);
            if house_value == 0 {
                return OpenPositionResult::Failed;
            }

            let Self {
                liquidity_manager,
                bias_tracker,
                ..
            } = self;

            let HouseLiquidityManager {
                free_liquidity,
                current_longs_reserve,
                current_shorts_reserve,
                current_net_debt,
                shorts_max_reserve_factor,
                longs_max_reserve_factor,
                total_deposit,
                ..
            } = liquidity_manager;
            //self.calculate_reserve_in_for_opening_position(price, long, max_pnl);

            let (max_reserve_for_bias, current_reserve_for_bias) = if long {
                (
                    apply_precision(*longs_max_reserve_factor, house_value),
                    current_longs_reserve,
                )
            } else {
                (
                    apply_precision(*shorts_max_reserve_factor, house_value),
                    current_shorts_reserve,
                )
            };

            if debt + added_reserve > *free_liquidity
                || added_reserve + *current_reserve_for_bias > max_reserve_for_bias
            {
                return OpenPositionResult::Failed;
            }
            // reduce free liquidity
            *free_liquidity = *free_liquidity - (added_reserve + debt);
            // increase current debt
            *current_net_debt += debt;
            // increase current debt for bias
            *current_reserve_for_bias += added_reserve;
            // increase total deposit
            *total_deposit += collateral;
            let position_open_interest = debt + collateral;

            let units = to_precision(position_open_interest, price);

            let params = UpdateBiasDetailsParamters {
                delta_net_debt_of_traders: debt as i128,
                delta_total_open_interest: position_open_interest as i128,
                delta_total_open_interest_dynamic: position_open_interest as i128,
                delta_total_units: units as i128,
                delta_net_reserve: max_pnl as i128,
            };

            bias_tracker.update_bias_details(params, long);

            let current_cumulative_funding_factor =
                self.get_cummulative_funding_factor_since_epoch(long);

            let current_cummulative_borrowing_factor =
                self.get_cummulative_borrowing_factor_since_epoch(long);

            // let price_impact = self.calculate_price_impact_open_position(position_open_interest);

            let position = Position {
                owner,
                collateral,
                long,
                debt,
                max_reserve: max_pnl,
                units,
                pre_cummulative_funding_factor: current_cumulative_funding_factor,
                pre_cummulative_borrowing_factor: current_cummulative_borrowing_factor,
            };
            OpenPositionResult::Settled { position }
        } else {
            // wait to fetch price
            let mut position = Position::default();

            position.long = long;
            position.collateral = collateral;
            position.debt = debt;
            position.max_reserve = max_pnl;

            OpenPositionResult::Waiting { position }
        }

        // get the amount
    }

    ///  
    pub fn close_position(
        &mut self,
        position: Position,
        acceptable_price_limit: u128,
    ) -> ClosePositionResult {
        let Position { long, .. } = position;
        let price_update = self
            .pricing_manager
            .get_price_within_interval(MAX_ALLOWED_PRICE_CHANGE_INTERVAL);

        if let Some(price) = price_update {
            // if closing a short and price is higher than acceptable price
            // if closing long ,and price is lower than acceptable price
            if (!long && price > acceptable_price_limit)
                || ((long) && price < acceptable_price_limit)
            {
                return ClosePositionResult::Failed;
            }
            let current_cummulative_funding_factor =
                self.get_cummulative_funding_factor_since_epoch(long);

            let current_cummulative_borrowing_factor =
                self.get_cummulative_borrowing_factor_since_epoch(long);

            let net_borrowing_factor =
                current_cummulative_borrowing_factor - position.pre_cummulative_borrowing_factor;
            let net_funding_factor =
                current_cummulative_funding_factor - position.pre_cummulative_funding_factor;

            let net_borrowing_fee = position.get_net_borrowing_fee(net_borrowing_factor);

            let net_funding_fee = position.get_net_funding_fee(net_funding_factor);

            let position_pnl = position.get_pnl(price);

            let Self {
                liquidity_manager,
                bias_tracker,
                ..
            } = self;

            let HouseLiquidityManager {
                total_deposit,
                free_liquidity,
                current_longs_reserve,
                current_shorts_reserve,
                current_net_debt,
                bad_debt,
                current_borrow_fees_owed,
                ..
            } = liquidity_manager;

            let (net_free_liquidity, mut collateral_out, house_bad_debt) = if net_funding_fee < 0 {
                position.close_position_with_net_negative_funding(
                    *free_liquidity,
                    net_funding_fee,
                    net_borrowing_fee,
                    position_pnl,
                )
            } else {
                position.close_position_with_net_positive_funding(
                    *free_liquidity,
                    net_funding_fee,
                    net_borrowing_fee,
                    position_pnl,
                )
            };

            *current_net_debt -= position.debt;
            *current_borrow_fees_owed -= net_borrowing_fee;

            // collateral out is only what is available in market
            collateral_out = collateral_out.min(*total_deposit);
            *total_deposit -= collateral_out;

            let bad_debt_removed = net_free_liquidity.min(*bad_debt);
            *bad_debt = (*bad_debt + house_bad_debt) - bad_debt_removed;
            *free_liquidity = net_free_liquidity - bad_debt_removed;

            if long {
                *current_longs_reserve -= position.max_reserve
            } else {
                *current_shorts_reserve -= position.max_reserve
            }

            let position_open_interest = position.open_interest();
            let delta_open_interest_dynamic =
                position_open_interest as i128 + net_funding_fee - (net_borrowing_fee as i128);

            let params = UpdateBiasDetailsParamters {
                delta_net_debt_of_traders: position.debt.neg(),
                delta_total_open_interest_dynamic: delta_open_interest_dynamic.neg(),
                delta_total_open_interest: position_open_interest.neg(),
                delta_total_units: position.units.neg(),
                delta_net_reserve: position.max_reserve.neg(),
            };

            bias_tracker.update_bias_details(params, long);

            ClosePositionResult::Settled {
                returns: collateral_out,
            }
        } else {
            ClosePositionResult::Waiting { position }
        }
    }

    pub fn _update_price(&mut self, price: u64, decimal: u32) {
        let Self {
            pricing_manager, ..
        } = self;

        let price_to_precision = to_precision(price as u128, 10u128.pow(decimal));
        pricing_manager.update_price(price_to_precision);
    }

    pub fn collect_borrowing_payment(&mut self) -> bool {
        let pricing_manager = self.pricing_manager;

        let price_update =
            pricing_manager.get_price_within_interval(MAX_ALLOWED_PRICE_CHANGE_INTERVAL);

        if let Some(price) = price_update {
            let pool_value = self._house_value(price);

            let Self {
                liquidity_manager,

                bias_tracker,
                ..
            } = self;

            let long_reserve = bias_tracker.longs.reserve_value(price, true);

            let short_reserve = bias_tracker.longs.reserve_value(price, false);

            let longs_borrow_factor_per_second = bias_tracker
                .longs
                .calculate_borrowing_factor_per_sec(pool_value, long_reserve);

            let shorts_borrow_factor_per_second = bias_tracker
                .shorts
                .calculate_borrowing_factor_per_sec(pool_value, short_reserve);

            let HouseLiquidityManager {
                last_time_since_borrow_fees_collected,
                current_borrow_fees_owed,
                ..
            } = liquidity_manager;

            let duration_in_secs = duration_in_secs(*last_time_since_borrow_fees_collected);

            let longs_borrow_payment = bias_tracker.longs.update_cumulative_borrowing_factor(
                longs_borrow_factor_per_second * duration_in_secs as u128,
            );

            let shorts_borrow_payement = bias_tracker.shorts.update_cumulative_borrowing_factor(
                shorts_borrow_factor_per_second * duration_in_secs as u128,
            );
            *last_time_since_borrow_fees_collected = time();
            *current_borrow_fees_owed += shorts_borrow_payement + longs_borrow_payment;
            return true;
        } else {
            return false;
        }

        // let price = pricing_manager
        //     .get_price_within_interval(MAX_ALLOWED_PRICE_CHANGE_INTERVAL)
    }

    /// Update funding
    ///
    /// functionis called on interval for payment of funding fees
    /// funding fee to paid after a duration is caluculated
    pub fn settle_funding_payment(&mut self) {
        let Self {
            funding_manager,
            bias_tracker,
            ..
        } = self;

        let Bias { longs, shorts, .. } = bias_tracker;

        let current_funding_factor_ps = funding_manager.current_funding_factor_ps();

        let duration = funding_manager._seconds_since_last_update();

        let majority_funding_factor = current_funding_factor_ps * duration as i128;

        let long_open_interest = longs.traders_open_interest();

        let short_open_interest = shorts.traders_open_interest();

        if current_funding_factor_ps <= 0 {
            // shorts pay long

            shorts.update_cumulative_funding_factor(majority_funding_factor);

            // long fee = (majority funding * short open interest)/ long_open_interest
            let longs_funding_factor = mul_div(
                majority_funding_factor.abs() as u128,
                short_open_interest,
                long_open_interest,
            ) * duration;
            //

            longs.update_cumulative_funding_factor(longs_funding_factor as i128);
        } else {
            //longs pay short
            longs.update_cumulative_funding_factor(majority_funding_factor * -1);

            // short fee  = (majority_funding_fee * long open interest) / short_open_interest
            let shorts_funding_factor = mul_div(
                majority_funding_factor.abs() as u128,
                long_open_interest,
                short_open_interest,
            ) * duration;
            shorts.update_cumulative_funding_factor(shorts_funding_factor as i128)
        }

        let current_long_short_diff = long_open_interest as i128 - short_open_interest as i128;

        let current_total_open_interest = long_open_interest + short_open_interest;

        funding_manager
            ._update_funding_factor_ps(current_long_short_diff, current_total_open_interest);
    }

    /// Calculates the current value of the
    pub fn _house_value(&mut self, price: u128) -> u128 {
        let house_value = (self.liquidity_manager.static_value() as i128
            - self.bias_tracker.net_house_pnl(price))
        .max(0);
        return house_value as u128;
    }
    pub fn base_asset(&self) -> AssetPricingDetails {
        self.asset_pricing_details.clone()
    }

    fn get_cummulative_funding_factor_since_epoch(&mut self, is_long_position: bool) -> i128 {
        let Self { bias_tracker, .. } = self;
        let Bias { longs, shorts, .. } = bias_tracker;
        if is_long_position {
            longs.cummulative_funding_factor_since_epoch()
        } else {
            shorts.cummulative_funding_factor_since_epoch()
        }
    }

    fn get_cummulative_borrowing_factor_since_epoch(&mut self, is_long_position: bool) -> u128 {
        let Self { bias_tracker, .. } = self;
        let Bias { longs, shorts, .. } = bias_tracker;
        if is_long_position {
            longs.cummulative_borrowing_factor_since_epcoh()
        } else {
            shorts.cummulative_borrowing_factor_since_epcoh()
        }
    }
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

fn duration_in_secs(start_time: u64) -> u64 {
    (time() - start_time) / 10u64.pow(9)
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

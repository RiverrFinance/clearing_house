use crate::market::components::bias::UpdateBiasDetailsParamters;
use crate::market::components::liquidity_manager::HouseLiquidityManager;
use crate::market::market_details::{MarketDetails, MarketState};
use candid::{CandidType, Deserialize};

use crate::constants::MAX_ALLOWED_PRICE_CHANGE_INTERVAL;
use crate::math::math::{apply_precision, to_precision};
use crate::open_position::open_position_params::OpenPositionParams;
use crate::position::position_details::PositionDetails;

#[cfg_attr(test, derive(Debug, Clone, Copy, PartialEq, Eq))]
#[derive(CandidType, Deserialize)]
pub enum OpenPositioninMarketResult {
    // Limit {acceptable_price:u128,position:Position},
    Settled { position: PositionDetails },
    Waiting { params: OpenPositionParams },
    Failed,
}

impl MarketDetails {
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
    pub fn open_position_in_market(
        &mut self,
        params: OpenPositionParams,
    ) -> OpenPositioninMarketResult {
        // Gets price within the required time interval
        let price_update = self
            .pricing_manager
            .get_price_within_interval(MAX_ALLOWED_PRICE_CHANGE_INTERVAL);

        self._open_position_in_market_with_price(params, price_update)
    }
    ///
    /// Fail checks condition
    ///
    /// if position is long and accpetable price islower than price
    ///
    ///
    /// Opening initial position
    /// Testing frame
    /// setting liquidity manager factors
    /// check changes
    /// max reserve for that hte respective bias increases by max pnl
    /// debt increases by that position debt  amount
    /// free liquidity reduces by debt and reserve amont
    ///
    /// biases expereince chnage too
    /// debt increases total_open_interest_dynamic increases same as open interest;
    /// position  pre cummulative and pre cumm borrowing factor is zero  
    pub fn _open_position_in_market_with_price(
        &mut self,
        params: OpenPositionParams,
        price_update: Option<u128>,
    ) -> OpenPositioninMarketResult {
        let OpenPositionParams {
            long,
            collateral,
            leverage_factor,
            acceptable_price_limit,
            reserve_factor,
            owner,
            ..
        } = params;
        if let Some(price) = price_update {
            if (long && price > acceptable_price_limit)
                || ((!long) && price < acceptable_price_limit)
            {
                return OpenPositioninMarketResult::Failed;
            }

            let market_state = (*self).state;

            let MarketState {
                max_leverage_factor,
                max_reserve_factor,
                ..
            } = market_state;

            if leverage_factor > max_leverage_factor || reserve_factor > max_reserve_factor {
                return OpenPositioninMarketResult::Failed;
            };

            let debt = apply_precision(leverage_factor, collateral) - collateral;

            let added_reserve = apply_precision(reserve_factor, collateral + debt); // 

            let house_value = self._house_value(price);

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
                return OpenPositioninMarketResult::Failed;
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
                delta_net_reserve: added_reserve as i128,
            };

            bias_tracker.update_bias_details(params, long);

            let current_cumulative_funding_factor =
                self.get_cummulative_funding_factor_since_epoch(long);

            let current_cummulative_borrowing_factor =
                self.get_cummulative_borrowing_factor_since_epoch(long);

            // let price_impact = self.calculate_price_impact_open_position(position_open_interest);

            let position = PositionDetails {
                owner,
                collateral,
                long,
                debt,
                max_reserve: added_reserve,
                units,
                pre_cummulative_funding_factor: current_cumulative_funding_factor,
                pre_cummulative_borrowing_factor: current_cummulative_borrowing_factor,
            };
            OpenPositioninMarketResult::Settled { position }
        } else {
            OpenPositioninMarketResult::Waiting { params }
        }
    }

    // get the amount
}

// #[cfg(test)]
// mod tests {
//     use candid::Principal;

//     use super::{MarketDetails, MarketState};
//     use crate::{
//         asset_management::AssetPricingDetails,
//         market::{
//             components::liquidity_manager::HouseLiquidityManager,
//             functions::open_position::OpenPositionResult,
//         },
//         math::math::FLOAT_PRECISION,
//     };

//     ///
//     /// Fail checks condition
//     ///
//     /// if position is long and accpetable price islower than price
//     ///
//     ///
//     /// Opening initial position
//     /// Testing frame
//     /// setting liquidity manager factors
//     /// check changes
//     /// max reserve for that hte respective bias increases by max pnl
//     /// debt increases by that position debt  amount
//     /// free liquidity reduces by debt and reserve amont
//     ///
//     /// biases expereince chnage too
//     /// debt increases total_open_interest_dynamic increases same as open interest;
//     /// position  pre cummulative and pre cumm borrowing factor is zero
//     ///
//     ///
//     fn initiate_market() -> MarketDetails {
//         let mut new_market = MarketDetails::default();

//         let max_leverage = 50 * FLOAT_PRECISION; //50x

//         let max_pnl_factor = 10 * FLOAT_PRECISION;

//         let market_state = MarketState {
//             max_leverage,
//             max_pnl_factor,
//             liquidation_factor: 10,
//         };
//         new_market.state = market_state;

//         let free_liquidity = 1_000_000 * FLOAT_PRECISION;

//         let liquidity_manager = HouseLiquidityManager {
//             total_liquidity_tokens_minted: free_liquidity,
//             total_deposit: free_liquidity,
//             current_longs_reserve: 0,
//             current_shorts_reserve: 0,
//             current_net_debt: 0,
//             bad_debt: 0,
//             free_liquidity,
//             current_borrow_fees_owed: 0,
//             longs_max_reserve_factor: (FLOAT_PRECISION / 10) * 3, // 30%,
//             shorts_max_reserve_factor: (FLOAT_PRECISION / 10) * 3,
//             last_time_since_borrow_fees_collected: 0,
//             liquidation_factor: FLOAT_PRECISION / 100, // 1%
//         };

//         new_market.liquidity_manager = liquidity_manager;

//         return new_market;
//     }

//     #[test]
//     fn open_position_fails_for_exceeding_price_limit() {
//         let higher_price = 123 * FLOAT_PRECISION;

//         let lower_price = 120 * FLOAT_PRECISION; // less than current price

//         let mut market = initiate_market();

//         let new_market = MarketDetails {
//             index_asset_pricing_details: AssetPricingDetails::default(),
//             token_identifier: "".to_string(),
//             ..market
//         };

//         // opening long positoon will fail for price_limit lower than current price
//         let result1 = market._open_position_with_price(
//             Principal::anonymous(),
//             0,
//             0,
//             true,
//             0,
//             lower_price,
//             Some(higher_price),
//         );

//         // for short ,it will fail for price limit lower than current price
//         let result2 = market._open_position_with_price(
//             Principal::anonymous(),
//             0,
//             0,
//             false,
//             0,
//             higher_price,
//             Some(lower_price),
//         );

//         // test not chnage haspned in the market
//         assert_eq!(new_market, market);
//         // test not chnage haspned in the market
//         assert_eq!(new_market, market);
//         //
//         assert_eq!(result1, OpenPositionResult::Failed);
//         assert_eq!(result2, OpenPositionResult::Failed)
//     }

//     #[test]
//     fn open_position_fails_for_max_leverage_exceeded() {
//         let current_price = 120 * FLOAT_PRECISION; // less than current price

//         let mut market = initiate_market();

//         let new_market = MarketDetails {
//             index_asset_pricing_details: AssetPricingDetails::default(),
//             token_identifier: "".to_string(),
//             ..market
//         };

//         let collateral = 600 * FLOAT_PRECISION;

//         let leverage = 60 * FLOAT_PRECISION; //60x  while set max leverage is 50 x;

//         let debt = super::apply_precision(leverage, collateral) - collateral;

//         let result = market._open_position_with_price(
//             Principal::anonymous(),
//             collateral,
//             debt,
//             true,
//             60 * FLOAT_PRECISION,
//             current_price,
//             Some(current_price),
//         );

//         assert_eq!(result, OpenPositionResult::Failed);
//         // no change occurs
//         assert_eq!(market, new_market);
//     }

//     #[test]
//     fn opening_position_adjusts_the_right_paramters() {
//         let current_price = 123 * FLOAT_PRECISION; // less than current price

//         let mut market = initiate_market();

//         let prev_bias_tracker_state = market.bias_tracker;

//         let prev_liquidity_manager_state = market.liquidity_manager;

//         let (
//             prev_total_open_interest,
//             prev_total_open_interest_dynamic,
//             prev_total_units,
//             prev_total_reserve,
//             prev_total_debt,
//         ) = prev_bias_tracker_state.longs.bias_parameters();

//         let HouseLiquidityManager {
//             total_deposit: prev_total_deposit,
//             current_longs_reserve: prev_longs_reserve,
//             current_net_debt: previous_net_debt,
//             free_liquidity: prev_free_liquidity,
//             ..
//         } = prev_liquidity_manager_state;

//         let collateral = 600 * FLOAT_PRECISION;

//         let leverage = 50 * FLOAT_PRECISION; //60x  while set max leverage is 50 x;

//         let debt = super::apply_precision(leverage, collateral) - collateral;

//         let max_pnl = 60 * FLOAT_PRECISION;

//         let result = market._open_position_with_price(
//             Principal::anonymous(),
//             collateral,
//             debt,
//             true,
//             max_pnl,
//             current_price,
//             Some(current_price),
//         );

//         let position = if let OpenPositionResult::Settled { position } = result {
//             position
//         } else {
//             panic!()
//         };

//         let position_units = position.units;

//         let current_bias_tracker_state = market.bias_tracker;

//         let (
//             current_total_open_interest,
//             current_total_open_interest_dynamic,
//             current_total_units,
//             current_total_reserve,
//             current_total_debt,
//         ) = current_bias_tracker_state.longs.bias_parameters();

//         assert_eq!(
//             current_total_open_interest,
//             prev_total_open_interest + collateral + debt
//         );
//         assert_eq!(
//             current_total_open_interest_dynamic,
//             prev_total_open_interest_dynamic + (collateral + debt) as i128
//         );
//         assert_eq!(current_total_debt, prev_total_debt + debt);

//         assert_eq!(current_total_reserve, prev_total_reserve + max_pnl);

//         assert_eq!(current_total_units, prev_total_units + position_units);

//         let current_liquidity_manager_state = market.liquidity_manager;

//         let HouseLiquidityManager {
//             total_deposit: current_total_deposit,
//             current_longs_reserve,
//             current_net_debt,
//             free_liquidity: current_free_liquidity,
//             ..
//         } = current_liquidity_manager_state;

//         assert_eq!(current_total_deposit, prev_total_deposit + collateral);
//         assert_eq!(current_longs_reserve, prev_longs_reserve + max_pnl);
//         assert_eq!(current_net_debt, previous_net_debt + debt);
//         assert_eq!(current_free_liquidity, prev_free_liquidity - debt - max_pnl)

//         //println!("the returned position is {:?}", result);

//         //assert_eq!(result, OpenPositionResult::Failed);
//         // no change occurs
//         //assert_eq!(market, new_market);
//     }
// }

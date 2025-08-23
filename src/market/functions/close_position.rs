use crate::close_position::close_position_result::ClosePositionResult;
use crate::market::components::liquidity_manager::HouseLiquidityManager;
use crate::market::market_details::MarketDetails;

use crate::market::components::bias::UpdateBiasDetailsParamters;

use crate::constants::MAX_ALLOWED_PRICE_CHANGE_INTERVAL;
use crate::math::math::Neg;
use crate::position::position_details::PositionDetails;
impl MarketDetails {
    pub fn close_position(
        &mut self,
        position: PositionDetails,
        acceptable_price_limit: u128,
    ) -> ClosePositionResult {
        let price_update = self
            .pricing_manager
            .get_price_within_interval(MAX_ALLOWED_PRICE_CHANGE_INTERVAL);
        self._close_position_with_price_hook(position, acceptable_price_limit, price_update)
    }

    pub fn _close_position_with_price_hook(
        &mut self,
        position: PositionDetails,
        acceptable_price_limit: u128,
        price_update: Option<u128>,
    ) -> ClosePositionResult {
        // self
        // .pricing_manager
        // .get_price_within_interval(MAX_ALLOWED_PRICE_CHANGE_INTERVAL);
        if let Some(price) = price_update {
            let PositionDetails { long, .. } = position;
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

            let net_borrowing_fee =
                position.get_net_borrowing_fee(current_cummulative_borrowing_factor);

            let net_funding_fee = position.get_net_funding_fee(current_cummulative_funding_factor);

            let position_pnl = position.get_pnl(price);

            let Self {
                liquidity_manager,
                bias_tracker,
                ..
            } = self;

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

            let HouseLiquidityManager {
                total_deposit,
                free_liquidity,
                current_longs_reserve,
                current_shorts_reserve,
                current_net_debt,
                bad_debt: current_house_bad_debt,
                current_borrow_fees_owed,
                ..
            } = liquidity_manager;

            let (net_free_liquidity, mut collateral_out, new_house_bad_debt) =
                if net_funding_fee < 0 {
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

            // removed positions share of changes to debt and borrow_fees even if it is not fully repaid
            *current_net_debt -= position.debt;
            *current_borrow_fees_owed -= net_borrowing_fee;

            // collateral out is only what is available in market
            collateral_out = collateral_out.min(*total_deposit);
            *total_deposit -= collateral_out;

            // bad debt is prioritised to be repaid before free liquidity is updated
            let bad_debt_removed = net_free_liquidity.min(*current_house_bad_debt);
            *current_house_bad_debt =
                (*current_house_bad_debt + new_house_bad_debt) - bad_debt_removed;
            *free_liquidity = net_free_liquidity - bad_debt_removed;

            if long {
                *current_longs_reserve -= position.max_reserve
            } else {
                *current_shorts_reserve -= position.max_reserve
            }

            ClosePositionResult::Settled {
                returns: collateral_out,
            }
        } else {
            ClosePositionResult::Waiting
        }
    }
}

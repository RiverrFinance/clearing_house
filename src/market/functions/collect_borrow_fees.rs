use ic_cdk::api::time;

use crate::constants::MAX_ALLOWED_PRICE_CHANGE_INTERVAL;
use crate::market::components::liquidity_manager::HouseLiquidityManager;
use crate::market::market_details::MarketDetails;
use crate::pricing_update_management::price_fetch::_fetch_price;
use crate::utils::duration_in_seconds;

impl MarketDetails {
    pub async fn collect_borrowing_payment(&mut self) -> bool {
        let duration_in_secs =
            |last_time_updated: u64| -> u64 { duration_in_seconds(last_time_updated) };

        self._collect_fees_after_duration(duration_in_secs).await
    }

    pub async fn _collect_fees_after_duration<F>(&mut self, duration_in_secs: F) -> bool
    where
        F: Fn(u64) -> u64,
    {
        let pricing_manager = (*self).pricing_manager;

        let price_update =
            pricing_manager.get_price_within_interval(MAX_ALLOWED_PRICE_CHANGE_INTERVAL);

        let price = match price_update {
            Some(price) => price,
            None => {
                let Ok((price, decimal)) = _fetch_price(self.index_asset_pricing_details()).await
                else {
                    return false;
                };

                self._update_price(price, decimal)
            }
        };

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

        let longs_current_borrow_fee_payment =
            bias_tracker.longs.update_cumulative_borrowing_factor(
                longs_borrow_factor_per_second * duration_in_secs as u128,
            );

        let shorts_current_borrow_fee_payement =
            bias_tracker.shorts.update_cumulative_borrowing_factor(
                shorts_borrow_factor_per_second * duration_in_secs as u128,
            );
        *last_time_since_borrow_fees_collected = time();
        *current_borrow_fees_owed +=
            shorts_current_borrow_fee_payement + longs_current_borrow_fee_payment;
        return true;
    }
}

use crate::market::components::bias::Bias;
use crate::market::components::funding_state::FundingState;
use crate::market::market_details::MarketDetails;
use crate::math::math::{Neg, mul_div};
use crate::utils::duration_in_seconds;

impl MarketDetails {
    /// Update funding
    ///
    /// functionis called on interval for payment of funding fees
    /// funding fee to paid after a duration is caluculated
    pub fn settle_funding_payment(&mut self) {
        let duration_in_secs = |start_time| duration_in_seconds(start_time);

        self._settle_funding_payment_after_duration(duration_in_secs);
    }

    pub fn _settle_funding_payment_after_duration<F>(&mut self, duration_in_secs: F)
    where
        F: Fn(u64) -> u64,
    {
        let Self {
            funding_state,
            bias_tracker,
            ..
        } = self;

        let Bias { longs, shorts } = bias_tracker;

        let FundingState {
            last_time_updated,
            current_funding_factor_ps,
            ..
        } = *funding_state;

        let duration = duration_in_secs(last_time_updated) as u128;

        let majority_funding_factor = current_funding_factor_ps * duration as i128;

        let long_open_interest = longs.traders_open_interest();

        let short_open_interest = shorts.traders_open_interest();

        if current_funding_factor_ps == 0 || long_open_interest == 0 || short_open_interest == 0 {
            return;
        }

        if current_funding_factor_ps <= 0 {
            // shorts pay long

            shorts.update_cumulative_funding_factor(majority_funding_factor);

            // long fee = (majority funding * short open interest)/ long_open_interest
            let longs_funding_factor = mul_div(
                majority_funding_factor.abs() as u128,
                short_open_interest,
                long_open_interest,
            );
            //

            longs.update_cumulative_funding_factor(longs_funding_factor as i128);
        } else {
            //longs pay short
            longs.update_cumulative_funding_factor(majority_funding_factor.neg());

            // short fee  = (majority_funding_fee * long open interest) / short_open_interest
            let shorts_funding_factor = mul_div(
                majority_funding_factor.abs() as u128,
                long_open_interest,
                short_open_interest,
            );
            shorts.update_cumulative_funding_factor(shorts_funding_factor as i128)
        }

        let current_long_short_diff = long_open_interest as i128 - short_open_interest as i128;

        let current_total_open_interest = long_open_interest + short_open_interest;

        funding_state
            ._update_funding_factor_ps(current_long_short_diff, current_total_open_interest);
    }
}

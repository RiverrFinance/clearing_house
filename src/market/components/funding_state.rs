use candid::CandidType;
use ic_cdk::api::time;
use serde::{Deserialize, Serialize};

use crate::math::math::{apply_exponent, apply_precision, bound_magnitude_signed, to_precision};

#[derive(PartialEq, Eq)]
enum FundingChangeType {
    Increase,
    Decrease,
    Nochange,
}

#[cfg_attr(test, derive(Debug, Clone, Copy, PartialEq, Eq))]
#[derive(Default, Deserialize, Serialize, CandidType)]
pub struct FundingState {
    /// Last time updated timetamp
    pub last_time_updated: u64,
    /// Next funding factor Per Second
    ///  to be paid after elapsed duration by either long or short
    /// Positive value for long paying shorts and
    /// negative value for shorts paying longg
    pub next_funding_factor_ps: i128,

    /// Min funding factor per second
    /// Serves as a lower threshold for funding factor per second
    pub min_funding_factor_ps: u128,
    /// Max funding factor per second
    pub max_funding_factor_ps: u128,

    /// Funding factor
    pub funding_factor: u128, // multiplied by 10*20
    pub funding_exponent_factor: u128,

    pub threshold_decrease_funding: u128,
    pub threshold_stable_funding: u128,
    /// funding increae per second
    pub funding_increase_factor_ps: u128,
    /// funding decrease per second
    pub funding_decrease_factor_ps: u128,
}

impl FundingState {
    pub fn current_funding_factor_ps(&self) -> i128 {
        return self.next_funding_factor_ps;
    }

    /// Update FUnding factor per second
    ///
    /// @dev updates the current funding factor based
    ///
    /// Funding fee per_sec is calculated as
    /// funding_fee_per_sec =(funding_factor * (longshort difference)^ (funding_factor_exponent))/ (total_open_interest)
    ///
    /// See README.md for more technical overview  info
    pub fn _update_funding_factor_ps(&mut self, long_short_diff: i128, total_open_interest: u128) {
        let Self {
            threshold_decrease_funding,
            threshold_stable_funding,
            funding_exponent_factor,
            funding_factor,
            funding_increase_factor_ps,
            funding_decrease_factor_ps,
            max_funding_factor_ps,
            min_funding_factor_ps,
            ..
        } = *self;

        assert!(total_open_interest > 0);

        let long_short_diff_mag = long_short_diff.abs() as u128;

        if long_short_diff == 0 {
            self.next_funding_factor_ps = 0
        }

        // (imbalance) ^ (funding_expoenent_factor)
        let long_short_after_exponent =
            apply_exponent(long_short_diff_mag, funding_exponent_factor);

        let long_short_diff_to_open_interest_factor =
            to_precision(long_short_after_exponent, total_open_interest);

        if funding_increase_factor_ps == 0 {
            // if there is no fundingIncreaseFactorPerSecond then return the static fundingFactor based on open interest difference
            let mut funding_factor_ps =
                apply_precision(long_short_diff_to_open_interest_factor, funding_factor);

            if funding_factor_ps > max_funding_factor_ps {
                funding_factor_ps = max_funding_factor_ps;
            }
            let sign = long_short_diff.abs() / long_short_diff;
            self.next_funding_factor_ps = funding_factor_ps as i128 * sign;

            return;
        }

        // current funding factor_ps
        // if positive then longs pay shorts
        // if ppsitive then shorts pay long
        let current_funding_factor_ps = self.next_funding_factor_ps;

        let current_funding_factor_ps_mag = current_funding_factor_ps.abs() as u128;

        let mut next_saved_funding_factor_ps = current_funding_factor_ps; // default to currentfunding factor

        let mut change_type = FundingChangeType::Nochange;

        // skew same direction is positive funding and longs more than shorts also when funding is negative shorts are more than longs
        let is_skew_same_direction = (current_funding_factor_ps > 0 && long_short_diff > 0)
            || (current_funding_factor_ps < 0 || long_short_diff < 0);

        if is_skew_same_direction {
            if long_short_diff_to_open_interest_factor > threshold_stable_funding {
                change_type = FundingChangeType::Increase
            } else if long_short_diff_to_open_interest_factor < threshold_decrease_funding {
                // if thresholdForDecreaseFunding is zero and diffUsdToOpenInterestFactor is also zero
                // then the fundingRateChangeType would be NoChange
                change_type = FundingChangeType::Decrease
            }
        } else {
            // if the skew has changed, then the funding should increase in the opposite direction
            change_type = FundingChangeType::Increase
        }

        if change_type == FundingChangeType::Increase {
            let mut increase_value = (apply_precision(
                long_short_diff_to_open_interest_factor,
                funding_increase_factor_ps,
            ) * self._seconds_since_last_update()) as i128;

            // if there are more longs than shorts, then the funding factor should increase
            // otherwise the funding factor per second should increase in the opposite direction / decrease
            if long_short_diff < 0 {
                increase_value = -increase_value;
            };

            next_saved_funding_factor_ps = current_funding_factor_ps + increase_value;
        }

        if change_type == FundingChangeType::Decrease && current_funding_factor_ps.abs() != 0 {
            let decrease_value = funding_decrease_factor_ps * self._seconds_since_last_update();

            if current_funding_factor_ps_mag <= decrease_value {
                // set the funding factor to 1 or -1 depending on the original savedFundingFactorPerSecond
                next_saved_funding_factor_ps =
                    current_funding_factor_ps.abs() / current_funding_factor_ps;
            } else {
                // reduce the original current funding factor per second while keeping the original sign of the current funding factor per second
                let sign = current_funding_factor_ps.abs() / current_funding_factor_ps;
                next_saved_funding_factor_ps =
                    (current_funding_factor_ps_mag - decrease_value) as i128 * sign;
            }
        };

        next_saved_funding_factor_ps = bound_magnitude_signed(
            next_saved_funding_factor_ps,
            min_funding_factor_ps,
            max_funding_factor_ps,
        );

        self.next_funding_factor_ps = next_saved_funding_factor_ps;
        self.last_time_updated = time();
    }

    pub fn _seconds_since_last_update(&self) -> u128 {
        ((time() - self.last_time_updated) / (10u64.pow(9u32))) as u128
    }
}

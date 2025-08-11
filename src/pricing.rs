use ic_cdk::api::time as now;
use serde::{Deserialize, Serialize};

use crate::math::math::{apply_exponent, apply_precision, diff};

type Time = u64;

#[derive(Default, Deserialize, Clone, Copy, Serialize)]
pub struct PricingManager {
    pub price: u128,
    pub last_fetched: Time,
    pub price_impact_exponent_factor: u128,
    pub positive_price_impact_factor: u128,
    pub negative_price_impact_factor: u128,
}

impl PricingManager {
    pub fn get_price_within_interval(&self, interval: u64) -> Option<u128> {
        if self.last_fetched + interval >= now() {
            return Some(self.price);
        } else {
            return None;
        }
    }
    pub fn update_price(&mut self, price: u128) {
        self.price = price;
        self.last_fetched = now()
    }
    // Price impact is calculated as:
    //
    // ```
    // (initial imbalance) ^ (price impact exponent) * (price impact factor / 2) - (next imbalance) ^ (price impact exponent) * (price impact factor / 2)
    // ``

    // @dev get the price impact USD if there is no crossover in balance
    // a crossover in balance is for example if the long open interest is larger
    // than the short open interest, and a short position is opened such that the
    // short open interest becomes larger than the long open interest
    // @param initialDiffUsd the initial difference in USD
    // @param nextDiffUsd the next difference in USD
    // @param impactFactor the impact factor
    // @param impactExponentFactor the impact exponent factor
    pub fn get_price_impact_for_same_side_rebalance(
        &self,
        initial_diff: u128,
        next_diff: u128,
    ) -> i128 {
        let Self {
            negative_price_impact_factor,
            positive_price_impact_factor,
            ..
        } = self;
        let has_positive_impact = next_diff < initial_diff;

        let impact_factor = if has_positive_impact {
            positive_price_impact_factor
        } else {
            negative_price_impact_factor
        };

        let delta_diff = diff(
            self.apply_impact_factor(initial_diff, *impact_factor),
            self.apply_impact_factor(next_diff, *impact_factor),
        );

        let price_impact = to_signed(delta_diff, has_positive_impact);

        return price_impact;
    }

    // @dev get the price impact USD if there is a crossover in balance
    // a crossover in balance is for example if the long open interest is larger
    // than the short open interest, and a short position is opened such that the
    // short open interest becomes larger than the long open interest
    // @param initialDiffUsd the initial difference in USD
    // @param nextDiffUsd the next difference in USD
    // @param hasPositiveImpact whether there is a positive impact on balance
    // @param impactFactor the impact factor
    // @param impactExponentFactor the impact exponent factor

    pub fn get_price_impact_for_crossover_rebalance(
        &self,
        initial_diff: u128,
        next_diff: u128,
    ) -> i128 {
        let Self {
            negative_price_impact_factor,
            positive_price_impact_factor,
            ..
        } = self;
        let positive_impact = self.apply_impact_factor(initial_diff, *positive_price_impact_factor);
        let negative_impact = self.apply_impact_factor(next_diff, *negative_price_impact_factor);

        let delta_diff = diff(positive_impact, negative_impact);

        let price_imapct = to_signed(delta_diff, positive_impact > negative_impact);

        return price_imapct;
    }

    // @dev apply the impact factor calculation to a USD diff value
    // @param diffUsd the difference in USD
    // @param impactFactor the impact factor
    // @param impactExponentFactor the impact exponent factor
    fn apply_impact_factor(&self, diff_usd: u128, impact_factor: u128) -> u128 {
        let Self {
            price_impact_exponent_factor,
            ..
        } = *self;
        let expoenent_value = apply_exponent(diff_usd, price_impact_exponent_factor);

        apply_precision(expoenent_value, impact_factor)
    }
}

fn to_signed(a: u128, is_positive: bool) -> i128 {
    if is_positive {
        a as i128
    } else {
        a as i128 * -1
    }
}

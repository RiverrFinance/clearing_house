use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::math::math::{Neg, apply_exponent, apply_precision, bound_signed, mul_div};

#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
#[derive(Default, Deserialize, Copy, Clone, Serialize, CandidType)]
pub struct Bias {
    pub longs: BiasDetails,
    pub shorts: BiasDetails,
}

#[derive(Default)]
pub struct UpdateBiasDetailsParamters {
    pub delta_total_open_interest: i128,
    pub delta_total_open_interest_dynamic: i128,
    pub delta_total_units: i128,
    pub delta_net_debt_of_traders: i128,
    pub delta_net_reserve: i128,
}

impl Bias {
    /// Update Bais Details

    pub fn update_bias_details(
        &mut self,
        params: UpdateBiasDetailsParamters,
        is_long_position: bool,
    ) {
        let Self { longs, shorts, .. } = self;
        if is_long_position {
            longs._update(params)
        } else {
            shorts._update(params)
        }
    }

    /// Long Short Open Interest Difference
    ///
    /// Calculates the difference between total long open interest and total short open interest
    pub fn long_short_open_interest_diff(&self) -> i128 {
        let Self { longs, shorts, .. } = self;
        longs.total_open_interest as i128 - shorts.total_open_interest as i128
    }

    /// Total Open Interest
    ///
    /// returns the total open interest i.e the sum of total long and short open interest
    pub fn total_open_interest(&self) -> u128 {
        let Self { longs, shorts, .. } = self;
        longs.total_open_interest + shorts.total_open_interest
    }

    /// House pnl
    ///
    /// calculates the net pnl of the all traders by adding the pnl of shorts wth the pnl of longs
    pub fn net_house_pnl(&self, price: u128) -> i128 {
        let pnl_of_longs = self.longs.house_pnl_by_specific_bias(price, true);

        let pnl_of_shorts = self.shorts.house_pnl_by_specific_bias(price, false);

        return pnl_of_shorts + pnl_of_longs;
    }
}

#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
#[derive(Default, Deserialize, Copy, Clone, Serialize, CandidType)]
pub struct BiasDetails {
    // Total amount in positions  in bias direction
    total_open_interest: u128,

    /// Total Open Interest Dynamic
    ///
    /// @dev Thsi is primarily used to track the maximum profit that the house can get
    ///  from traders losse after positon funding fees and borrowing fees have been paid
    /// the value would always reduce for borrowing factor update
    /// the value would increase for a positive funding factor  and reduce for a negative funding factor update
    /// bpunded below by net debt
    total_open_interest_dynamic: i128,

    // Total amount in tokens or units bought for longs  or sold for shorts
    total_units: u128,
    /// Total Reserve
    ///
    // Total amounts in position backed by pool, amount needed to ensure
    // also the sum of the maximum profit for all positions currently opened
    total_reserve: u128,
    /// The total debt by traders
    ///
    /// tracks the debt in taking levarage @dev different from net debt of tokens dynamic
    total_debt_of_traders: u128,

    /// the next borrowing fundig factor to be paid
    current_borrowing_factor: u128,

    /// Cummulative fuding factor since epoch
    ///
    cummulative_funding_factor_since_epoch: i128,
    /// Cummulative fuding factor since epoch
    ///
    cummulative_borrowing_factor_since_epoch: u128,
    /// Borrowing exponent factor
    ///
    /// Configurable parameter utilized for calculating borrowing factor
    borrowing_exponent_factor_: u128,
    base_borrowing_factor: u128,
}

impl BiasDetails {
    pub fn _update(&mut self, params: UpdateBiasDetailsParamters) {
        let Self {
            total_open_interest: net_open_interest,
            total_units,
            total_reserve: net_reserve,
            total_debt_of_traders,
            ..
        } = *self;

        let UpdateBiasDetailsParamters {
            delta_total_open_interest_dynamic,
            delta_net_debt_of_traders,
            delta_net_reserve,
            delta_total_open_interest,
            delta_total_units,
        } = params;

        self.total_open_interest =
            ((net_open_interest as i128) + delta_total_open_interest) as u128;

        self.total_units = ((total_units as i128) + delta_total_units) as u128;

        self.total_open_interest_dynamic += delta_total_open_interest_dynamic;

        self.total_debt_of_traders =
            ((total_debt_of_traders as i128) + delta_net_debt_of_traders) as u128;
        self.total_reserve = ((net_reserve as i128) + delta_net_reserve) as u128;
    }

    pub fn cummulative_funding_factor_since_epoch(&self) -> i128 {
        self.cummulative_funding_factor_since_epoch
    }

    pub fn cummulative_borrowing_factor_since_epcoh(&self) -> u128 {
        self.cummulative_borrowing_factor_since_epoch
    }

    pub fn house_pnl_by_specific_bias(&self, price: u128, is_long: bool) -> i128 {
        let Self {
            total_open_interest,
            total_units,
            total_reserve,
            total_open_interest_dynamic,
            total_debt_of_traders,
            ..
        } = *self;
        let sign = if is_long { 1 } else { -1 };

        let reduced_pnl_by_bad_debt = total_open_interest_dynamic.min(0);
        let minimum_pnl = total_reserve.neg() + reduced_pnl_by_bad_debt;

        let house_pnl = bound_signed(
            (total_open_interest as i128) - (apply_precision(total_units, price) as i128) * sign,
            minimum_pnl,
            total_open_interest_dynamic - (total_debt_of_traders as i128),
        );

        house_pnl
    }

    /// Traders PNL by Bias calculation
    ///
    /// Calculates the current pnl of  traders in a particular bias direction
    /// @dev at any point in time this figure is bounded
    /// bounded above by total_reserve as that as that is the max the house can lose
    /// bounded below by  the difference between  total_open_interest_dynamic (see BiasDetails) and net_debt
    pub fn traders_pnl_for_specific_bias(&self, price: u128, is_long: bool) -> i128 {
        let Self {
            total_open_interest,
            total_units,
            total_reserve,
            total_open_interest_dynamic,
            total_debt_of_traders,
            ..
        } = *self;
        let sign = if is_long { 1 } else { -1 };

        let traders_pnl = bound_signed(
            (apply_precision(total_units, price) as i128 - total_open_interest as i128) * sign,
            (total_open_interest_dynamic - (total_debt_of_traders) as i128).min(0),
            total_reserve as i128,
        );
        return traders_pnl;
    }

    pub fn reserve_value(&self, price: u128, is_long: bool) -> u128 {
        let Self {
            total_open_interest_dynamic,
            ..
        } = *self;
        let traders_pnl = self.traders_pnl_for_specific_bias(price, is_long);

        let traders_net_payout = (total_open_interest_dynamic as i128 + traders_pnl) as u128;

        return traders_net_payout;
    }

    pub fn calculate_borrowing_factor_per_sec(
        &self,
        pool_value: u128,
        reserve_value: u128,
    ) -> u128 {
        let Self {
            base_borrowing_factor,
            borrowing_exponent_factor_,
            ..
        } = *self;
        let reserve_after_expoent = apply_exponent(reserve_value, borrowing_exponent_factor_);

        let borrowing_factor_per_sec =
            mul_div(base_borrowing_factor, reserve_after_expoent, pool_value);

        return borrowing_factor_per_sec;
    }

    /// Update Cummulative Borrowing factor
    ///
    /// @dev this  is simllar to the Update Cummulative funding factor with slight difference explained below
    /// For cummulative funding factor ,it is updated during time of collection as the factor value has been precalculated and stored in the liquidity_manager for Market
    /// For updating cummulative  borrowing factor ,this  precalculated value is stored in the respective biases and  when
    /// this function is called with new factor to be paid after durtion as argument ,the current payment is calculated and subtracted from open interest and returned from function  
    pub fn update_cumulative_borrowing_factor(&mut self, delta_cfr: u128) -> u128 {
        let Self {
            current_borrowing_factor: previous_borrowing_factor,
            ..
        } = *self;
        // amount paid
        let value = apply_precision(previous_borrowing_factor, self.total_open_interest);

        // net open_interest when trader has lost all collateral
        self.total_open_interest_dynamic = self.total_open_interest_dynamic - value as i128;

        self.cummulative_borrowing_factor_since_epoch += previous_borrowing_factor;

        self.current_borrowing_factor = delta_cfr;
        return value;
    }

    /// Update Cummultive Funding factor
    ///
    /// @dev this function and the Update cummulative Borrowing Factor( see Below)  both update the cummultive funding and borrowing factor respectively  
    ///
    /// both function also update total_open interest dynamic
    pub fn update_cumulative_funding_factor(&mut self, delta_cfr: i128) {
        // if funding factor is positive ,reduce  debt by trader and if negative increase debt by trader

        let sign = if delta_cfr > 0 { 1 } else { -1 };

        // if funding factor is positive ,reduce debt
        // if funding factor is negative ,increase funding factor

        let value = apply_precision(delta_cfr.abs() as u128, self.total_open_interest) as i128;

        self.total_open_interest_dynamic = self.total_open_interest_dynamic + (value * sign);

        self.cummulative_funding_factor_since_epoch += delta_cfr;
    }
    pub fn traders_open_interest(&self) -> u128 {
        self.total_open_interest
    }

    pub fn bias_parameters(&self) -> (u128, i128, u128, u128, u128) {
        return (
            self.total_open_interest,
            self.total_open_interest_dynamic,
            self.total_units,
            self.total_reserve,
            self.total_debt_of_traders,
        );
    }
}

use serde::{Deserialize, Serialize};

use crate::math::{apply_precision, bound_below_signed, bound_signed};

#[derive(Default, Deserialize, Serialize)]
pub struct Bias {
    pub long: BiasDetails,
    pub short: BiasDetails,
}

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
        let Self { long, short } = self;
        if is_long_position {
            long._update(params)
        } else {
            short._update(params)
        }
    }
    // fn current_unrealised_reserve_for_longs(&self, price: u128) -> u128 {
    //     let (_, _, net_reserve, _) = self.long.bias_parameters();

    //     let house_pnl_by_longs = self.house_pnl_by_longs(price);

    //     return (house_pnl_by_longs + net_reserve as i128) as u128;
    // }

    // fn current_unrealised_reserve_for_shorts(&self, price: u128) -> u128 {
    //     let (_, _, net_reserve_, _) = self.short.bias_parameters();

    //     let house_pnl_by_shorts = self.house_pnl_by_shorts(price);

    //     return (house_pnl_by_shorts + net_reserve_ as i128) as u128;
    // }

    /// Long Short Open Interest Difference
    ///
    /// Calculates the difference between total long open interest and total short open interest
    pub fn long_short_open_interest_diff(&self) -> i128 {
        let Self { long, short } = self;
        long.total_open_interest as i128 - short.total_open_interest as i128
    }

    /// Total Open Interest
    /// returns the total open interest i.e the sum of total long and short open interest
    pub fn total_open_interest(&self) -> u128 {
        let Self { long, short } = self;
        long.total_open_interest + short.total_open_interest
    }

    pub fn house_pnl(&self, price: u128) -> i128 {
        //for shorts the  hpouse pnl is positive for price increase  and negative for price decrease ;
        let house_pnl_by_shorts = self.house_pnl_by_shorts(price);

        let house_pnl_by_longs = self.house_pnl_by_longs(price);

        return house_pnl_by_shorts + house_pnl_by_longs;
    }

    /// House PNL by longs
    ///
    /// Calculates the current pnl of the house by long positions i.e acting as a counter party to long positions
    /// @dev at any point in time this figure is bounded
    /// bounded lower by net_reserve as that as that is the max the house can lose
    /// bounded above by  based on certain conditions
    /// if net_debt_dynamic is greater  than 0
    /// the upper pnl bound is total net_debt_dynamic
    /// if net_debt_dynamic is greater is less than 0
    /// meaning all positions can pay off debt with the funding received
    fn house_pnl_by_longs(&self, price: u128) -> i128 {
        let (total_open_interest, total_open_interest_dynamic, total_units, net_reserve, net_debt) =
            self.short.bias_parameters();

        bound_signed(
            total_open_interest as i128 - apply_precision(total_units, price) as i128,
            net_reserve as i128 * -1,
            (total_open_interest_dynamic - net_debt) as i128,
        )
    }

    /// House PNL by Shorts
    ///
    /// Calculates the current pnl of the house by short positions i.e acting as a counter party to short positions
    /// @dev at any point in time this figure is bounded
    /// bounded lower by net_reserve as that as that is the max thew house can lose
    /// bounded above by the difference between total open interest for shorts  and current net_debt_owed_by long_traders
    fn house_pnl_by_shorts(&self, price: u128) -> i128 {
        let (total_open_interest, total_open_interest_dynamic, total_units, net_reserve, net_debt) =
            self.short.bias_parameters();

        //for shorts the  house pnl is positive for price increase  and negative for price decrease ;

        bound_signed(
            apply_precision(total_units, price) as i128 - total_open_interest as i128,
            net_reserve as i128 * -1,
            (total_open_interest_dynamic - net_debt) as i128,
        )
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct BiasDetails {
    // Total amount in positions  in bias direction
    total_open_interest: u128,

    total_open_interest_dynamic: u128,

    // Total amount in tokens or units bought and sold
    total_units: u128,

    // Total amounts in position backed by pool, amount need to ensure
    // also the sum of the maximum profit for all positions currently opened
    total_reserve: u128,
    /// The net debt by traders
    /// tracks the debt in taking levarage @dev different from net debt of tokens dynamic
    total_debt_of_traders: u128,

    // ///Net debt of Traders  Dynamic
    // ///
    // /// tracks the current amount owed by traders fluidly
    // /// @dev Note this does not only trackdebt on opening position with leverage ,it also tracks paid borrowing fee also fee paid as funding fee
    // /// used primarily to determine ,
    // net_debt_of_traders_dynamic: i128,
    ///  Next funding factor
    /// the  next funding factor to paid or earned
    next_funding_factor: i128,

    /// the next borrowing fundig factor to be paid
    next_borrowing_factor: u128,

    /// Cummulative fuding factor since epoch
    cummulative_funding_factor_since_epoch: i128,
    /// Cummulative fuding factor since epoch
    cummulative_borrowing_factor_since_epoch: u128,
}

impl BiasDetails {
    pub fn _update(&mut self, params: UpdateBiasDetailsParamters) {
        let Self {
            total_open_interest: net_open_interest,
            total_units,
            total_reserve: net_reserve,
            total_open_interest_dynamic,
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

        self.total_open_interest_dynamic =
            ((total_open_interest_dynamic as i128) + delta_total_open_interest_dynamic) as u128;

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

    // pub fn cummulative_funding_factor_since_epoch(&self) -> i128 {
    //     self.cummulative_funding_factor
    // }

    pub fn update_cumulative_funding_factor(&mut self, delta_cfr: i128) {
        let Self {
            next_funding_factor,
            ..
        } = *self;

        // if funding factor is positive ,reduce  debt by trader and if negative increase debt by trader

        let sign = if next_funding_factor > 0 { 1 } else { -1 };

        // if funding factor is positive ,reduce debt
        // if funding factor is negative ,increase funding factor

        let value =
            apply_precision(next_funding_factor.abs() as u128, self.total_open_interest) as i128;

        self.total_open_interest_dynamic = bound_below_signed(
            self.total_open_interest_dynamic as i128 + (value * sign),
            self.total_debt_of_traders as i128,
        )
        .abs() as u128;

        // Update tje next funding factor
        self.next_funding_factor = delta_cfr;
        self.cummulative_funding_factor_since_epoch += delta_cfr;
    }
    pub fn update_cumulative_borrowing_factor(&mut self, delta_cfr: u128) {
        let Self {
            next_borrowing_factor,
            ..
        } = *self;

        let value = apply_precision(next_borrowing_factor, self.total_open_interest);

        // bounds the next debt of trader by

        // net open_interest when trader has lost all collateral
        self.total_open_interest_dynamic = bound_below_signed(
            self.total_open_interest_dynamic as i128 - value as i128,
            self.total_debt_of_traders as i128,
        )
        .abs() as u128;

        self.next_borrowing_factor = delta_cfr;

        self.cummulative_borrowing_factor_since_epoch += delta_cfr;
    }
    pub fn traders_open_interest(&self) -> u128 {
        self.total_open_interest
    }

    /// to get traders current net pnl
    ///
    /// get the amount the trader net units purchased
    /// for longs value of purchased unit - value of open interest
    /// for shorts value of open interest - purchased unit

    pub fn bias_parameters(&self) -> (u128, u128, u128, u128, u128) {
        return (
            self.total_open_interest,
            self.total_open_interest_dynamic,
            self.total_units,
            self.total_reserve,
            self.total_debt_of_traders,
        );
    }
}

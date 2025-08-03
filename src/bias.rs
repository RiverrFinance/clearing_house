use bincode::{Decode, Encode};

use crate::math::apply_precision;

#[derive(Encode, Decode, Default)]
pub struct Bias {
    pub long: BiasDetails,
    pub short: BiasDetails,
}

impl Bias {
    pub fn update_bias_details(
        &mut self,
        delta_toi: i128,
        delta_hoi: i128,
        delta_hps: i128,
        is_long_position: bool,
    ) {
        let Self { long, short } = self;
        if is_long_position {
            long._update(delta_toi, delta_hoi, delta_hps);
        } else {
            short._update(delta_toi, delta_hoi, delta_hps);
        }
    }
    pub fn current_unrealised_reserve_for_longs(&self, price: u128) -> u128 {
        let (longs_maximum_profit, _) = self.long.house_paramters();

        let house_pnl_by_longs = self.house_pnl_by_longs(price);

        return (house_pnl_by_longs + longs_maximum_profit as i128) as u128;
    }

    pub fn current_unrealised_reserve_for_shorts(&self, price: u128) -> u128 {
        let (shorts_maximum_profit, _) = self.short.house_paramters();

        let house_pnl_by_shorts = self.house_pnl_by_shorts(price);

        return (house_pnl_by_shorts + shorts_maximum_profit as i128) as u128;
    }

    pub fn long_short_open_interest_diff(&self) -> i128 {
        let Self { long, short } = self;
        long.traders_open_interest as i128 - short.traders_open_interest as i128
    }

    pub fn total_open_interest(&self) -> u128 {
        let Self { long, short } = self;
        long.traders_open_interest + short.traders_open_interest
    }

    pub fn house_pnl(&self, price: u128) -> i128 {
        //for shorts the  hpouse pnl is positive for price increase  and negative for price decrease ;
        let house_pnl_by_shorts = self.house_pnl_by_shorts(price);

        let house_pnl_by_longs = self.house_pnl_by_longs(price);

        return house_pnl_by_shorts + house_pnl_by_longs;
    }

    pub fn house_pnl_by_longs(&self, price: u128) -> i128 {
        let (longs_maximum_profit, longs_maximum_size) = self.long.house_paramters();

        longs_maximum_profit as i128 - apply_precision(longs_maximum_size, price) as i128
    }

    pub fn house_pnl_by_shorts(&self, price: u128) -> i128 {
        let (shorts_maximum_profit, shorts_maximum_size) = self.short.house_paramters();

        //for shorts the  hpouse pnl is positive for price increase  and negative for price decrease ;
        apply_precision(shorts_maximum_size, price) as i128 - shorts_maximum_profit as i128
    }
}

#[derive(Encode, Decode, Default)]
pub struct BiasDetails {
    // Total amount in positions  inbias direction
    traders_open_interest: u128,

    // Total amounts in position backed by pool, amount need to ensure
    // also the sum of the maximum profit for all positions currently opened
    reserve_amount: u128,
    // Total amunt of assets i positions backed by pool
    //  this is the sum of the maximum position size of all  positions currently opened in that bias direction
    house_position_units: u128,

    /// Cummulative fuding factor since epoch
    cummulative_funding_factor: i128,
    /// Cummulative fuding factor since epoch
    cummulativw_borrowing_factor: u128,
}

impl BiasDetails {
    pub fn _update(&mut self, delta_toi: i128, delta_ra: i128, delta_hpu: i128) {
        let Self {
            traders_open_interest,
            reserve_amount,
            house_position_units,
            ..
        } = *self;

        self.traders_open_interest = ((traders_open_interest as i128) + delta_toi) as u128;
        self.reserve_amount = ((reserve_amount as i128) + delta_ra) as u128;
        self.house_position_units = ((house_position_units as i128) + delta_hpu) as u128;
    }

    pub fn update_cumulative_funding_factor(&mut self, delta_cfr: i128) {
        self.cummulative_funding_factor += delta_cfr;
    }
    pub fn update_cumulative_borrowing_factor(&mut self, delta_cfr: i128) {
        self.cummulative_funding_factor += delta_cfr;
    }
    pub fn traders_open_interest(&self) -> u128 {
        self.traders_open_interest
    }

    pub fn house_paramters(&self) -> (u128, u128) {
        return (self.reserve_amount, self.house_position_units);
    }
}

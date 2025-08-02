use bincode::{Decode, Encode};

use crate::math::apply_precision;

#[derive(Encode, Decode, Default)]
pub struct Bias {
    pub long: BiasDetails,
    pub short: BiasDetails,
}

impl Bias {
    pub fn long_short_diff(&self) -> i128 {
        let Self { long, short } = self;
        long.traders_open_interest as i128 - short.traders_open_interest as i128
    }

    pub fn total_open_interest(&self) -> u128 {
        let Self { long, short } = self;
        long.traders_open_interest + short.traders_open_interest
    }

    pub fn house_pnl(&self, price: u128) -> i128 {
        let Self { long, short } = self;

        // traders long position is house short position and vice versa
        //i.e house takes oposite side of trades

        let (shorts_maximum_profit, shorts_maximum_size) = short.house_paramters();

        let (longs_maximum_profit, longs_maximum_size) = long.house_paramters();

        //for shorts the  hpouse pnl is positive for price increase  and negative for price decrease ;
        let house_pnl_by_shorts =
            apply_precision(shorts_maximum_size, price) as i128 - shorts_maximum_profit as i128;

        let house_pnl_by_longs =
            longs_maximum_profit as i128 - apply_precision(longs_maximum_size, price) as i128;

        return house_pnl_by_shorts + house_pnl_by_longs;
    }
}

#[derive(Encode, Decode, Default)]
pub struct BiasDetails {
    // Total amount in positions  inbias direction
    traders_open_interest: u128,

    // Total amounts in position backed by pool
    // also the sum of the maximum profit for all positions currently opened
    maximum_profit: u128,
    // Total amunt of assets i positions backed by pool
    //  this is the sum of the maximum position size of all  positions currently opened in that bias direction
    house_position_size: u128,

    /// Cummulative fuding factor since epoch
    cummulative_funding_factor: i128,
    /// Cummulative fuding factor since epoch
    cummulativw_borrowing_factor: u128,
}

impl BiasDetails {
    pub fn _update(&mut self, delta_toi: i128, delta_hoi: i128, delta_hps: i128) {
        let Self {
            traders_open_interest,
            maximum_profit,
            house_position_size,
            ..
        } = *self;

        self.traders_open_interest = ((traders_open_interest as i128) + delta_toi) as u128;
        self.maximum_profit = ((maximum_profit as i128) + delta_hoi) as u128;
        self.house_position_size = ((house_position_size as i128) + delta_hps) as u128;
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
        return (self.maximum_profit, self.house_position_size);
    }
}

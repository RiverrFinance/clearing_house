use bincode::{self, Decode, Encode};
use core::time;
use ic_cdk::api::time;

use rust_decimal::{Decimal, dec};
use std::borrow::Cow;

use ic_stable_structures::storable::{Bound, Storable};

use crate::{
    funding::FundingManager,
    math::{_percentage, mul_div},
    types::Asset,
    vault::Vault,
};

type Time = u64;
type Amount = u128;

const ONE_SECOND: u64 = 1_000_000_000;

const ONE_HOUR: u64 = 3_600_000_000_000;

#[derive(Encode, Decode, Default)]
pub struct Price {
    pub price: u64,
    pub decimals: u32,
    pub last_fetched: Time,
}

impl Price {
    fn within_bound(&self, entry_price: u64, slippage: u64, long: bool) -> bool {
        let perc = _percentage(slippage, entry_price);

        if long {
            return entry_price + perc >= self.price;
        } else {
            let fact = Decimal::from_i128_with_scale(100, 6);
            let cam = fact / dec!(20);
            return entry_price - perc <= self.price * u64::MAX;
        }
    }
}

#[derive(Encode, Decode, Default)]
pub struct MarketState {
    pub max_leverage_x10: u8,
    pub max_pnl: u64,
    pub min_collateral: u128,
    pub execution_fee: u128,
    pub cummulative_borrowing_rate: u64,
    pub price_impact_exponent: u8,
    pub price_impact_factor: u8,
}

#[derive(Encode, Decode, Default)]
pub struct MarketDetails {
    pub base_asset: Asset,
    pub bias_tracker: BiasTracker,
    pub funding_manager: FundingManager,
    pub price: Price,
    pub state: MarketState,
}

impl MarketDetails {
    /// Update funding
    ///
    /// functionis called on interval for payment of funding fees
    /// funding fee to paid after a duration is caluculated
    pub fn update_funding(&mut self) {
        let Self {
            funding_manager,
            bias_tracker,
            ..
        } = self;

        let BiasTracker { long, short } = bias_tracker;

        let current_funding_factor_ps = funding_manager.current_funding_factor_ps();

        let duration = funding_manager._seconds_since_last_update();

        let majority_funding_fee = current_funding_factor_ps * duration as i128;

        let long_open_interest = long.traders_open_interest();

        let short_open_interest = short.traders_open_interest();

        if current_funding_factor_ps <= 0 {
            // shorts pay long

            short.update_cumulative_funding_factor(majority_funding_fee);
            let long_funding_fee = mul_div(
                majority_funding_fee.abs() as u128,
                long_open_interest,
                short_open_interest,
            ) * duration;
            long.update_cumulative_funding_factor(long_funding_fee as i128);
        } else {
            long.update_cumulative_funding_factor(majority_funding_fee);
            let short_funding_fee = mul_div(
                majority_funding_fee.abs() as u128,
                short_open_interest,
                long_open_interest,
            ) * duration;
            short.update_cumulative_funding_factor(short_funding_fee as i128)
        }

        let long_short_diff = long_open_interest as i128 - short_open_interest as i128;

        let total_open_interest = long_open_interest + short_open_interest;

        funding_manager._update_funding_factor_ps(long_short_diff, total_open_interest);
    }
    pub fn open_positon_at_current_price() {}
}
//price impact = (initial USD difference) ^ (price impact exponent) * (price impact factor) - (next USD difference) ^ (price impact exponent) * (price impact factor)

impl Storable for MarketDetails {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let mut slice = [0u8; 500];
        let length =
            bincode::encode_into_slice(self, &mut slice, bincode::config::standard()).unwrap();

        let slice = &slice[..length];
        Cow::Owned(slice.to_vec())
    }

    /// Converts the element into an owned byte vector.
    ///
    /// This method consumes `self` and avoids cloning when possible.
    fn into_bytes(self) -> Vec<u8> {
        let mut slice = [0u8; 500];
        let length =
            bincode::encode_into_slice(self, &mut slice, bincode::config::standard()).unwrap();

        let slice = &slice[..length];
        slice.to_vec()
    }

    /// Converts bytes into an element.
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        bincode::decode_from_slice(bytes.as_ref(), bincode::config::standard())
            .expect("Failed to decode MarketDetails")
            .0
    }

    /// The size bounds of the type.
    const BOUND: Bound = Bound::Unbounded;

    /// Like `to_bytes`, but checks that bytes conform to declared bounds.
    fn to_bytes_checked(&self) -> Cow<'_, [u8]> {
        let bytes = self.to_bytes();
        Self::check_bounds(&bytes);
        bytes
    }

    /// Like `into_bytes`, but checks that bytes conform to declared bounds.
    fn into_bytes_checked(self) -> Vec<u8>
    where
        Self: Sized,
    {
        let bytes = self.into_bytes();
        Self::check_bounds(&bytes);
        bytes
    }

    #[inline]
    fn check_bounds(bytes: &[u8]) {
        if let Bound::Bounded {
            max_size,
            is_fixed_size,
        } = Self::BOUND
        {
            let actual = bytes.len();
            if is_fixed_size {
                assert_eq!(
                    actual, max_size as usize,
                    "expected a fixed-size element with length {} bytes, but found {} bytes",
                    max_size, actual
                );
            } else {
                assert!(
                    actual <= max_size as usize,
                    "expected an element with length <= {} bytes, but found {} bytes",
                    max_size,
                    actual
                );
            }
        }
    }
}

#[derive(Encode, Decode, Default)]
pub struct BiasTracker {
    pub long: BiasDetails,
    pub short: BiasDetails,
}

impl BiasTracker {
    pub fn long_short_diff(&self) -> i128 {
        let Self { long, short } = self;
        long.traders_open_interest as i128 - short.traders_open_interest as i128
    }

    pub fn total_open_interest(&self) -> u128 {
        let Self { long, short } = self;
        long.traders_open_interest + short.traders_open_interest
    }

    // pub fn add_volume(&mut self, delta: Amount, long: bool) -> Amount {
    //     if long {
    //         self.long.add_volume(delta)
    //     } else {
    //         self.short.add_volume(delta)
    //     }
    // }

    // pub fn remove_volume(&mut self, delta: Amount, long: bool) -> Amount {
    //     if long {
    //         self.long.remove_volume(delta)
    //     } else {
    //         self.short.remove_volume(delta)
    //     }
    // }
}

#[derive(Encode, Decode, Default)]
pub struct BiasDetails {
    traders_open_interest: u128,
    house_open_interest: u128,
    house_position_size: u128,
    cummulative_funding_factor: i128,
    cummulativw_borrowing_factor: u128,
}

impl BiasDetails {
    fn update(&mut self, delta_toi: i128, delta_hoi: i128, delta_hps: i128) {
        let Self {
            traders_open_interest,
            house_open_interest,
            house_position_size,
            ..
        } = *self;

        self.traders_open_interest = ((traders_open_interest as i128) + delta_toi) as u128;
        self.house_open_interest = ((house_open_interest as i128) + delta_hoi) as u128;
        self.house_position_size = ((house_position_size as i128) + delta_hps) as u128;
    }

    fn update_cumulative_funding_factor(&mut self, delta_cfr: i128) {
        self.cummulative_funding_factor += delta_cfr;
    }
    fn traders_open_interest(&self) -> u128 {
        self.traders_open_interest
    }
}

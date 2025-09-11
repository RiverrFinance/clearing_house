use std::borrow::Cow;

use candid::{CandidType, Principal};
use ic_stable_structures::{Storable, storable::Bound};
use serde::{Deserialize, Serialize};

use crate::math::math::{apply_precision, bound_above_signed};

/// Position

#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
#[derive(Copy, Clone, Deserialize, CandidType, Serialize)]
pub struct PositionDetails {
    /// Owner
    ///
    /// position owner
    pub owner: Principal,
    /// Collateral
    ///
    /// collateral put for opening position
    pub collateral: u128,
    /// Debt
    pub debt: u128,
    ///Long
    ///
    /// true if long and false if short
    pub long: bool,
    /// Units
    ///
    /// The amount in base asset bought (longs) or sold (shorts) for that position
    pub units: u128,
    /// Max Reserve
    ///
    /// the max reserve for the position
    pub max_reserve: u128,

    /// cummulttive funding factor since genesis at time  of opening or updating position
    pub pre_cummulative_funding_factor: i128,
    /// cummulttive funding factor since epoch  at point of opening or updating position
    pub pre_cummulative_borrowing_factor: u128,
}

impl PositionDetails {
    pub fn get_pnl(&self, price: u128) -> i128 {
        let Self {
            long,
            units,
            collateral,
            debt,
            max_reserve,
            ..
        } = *self;

        let units_value = apply_precision(price, units) as i128;

        let pnl = units_value - (collateral + debt) as i128;
        let sign = if long { 1 } else { -1 };
        bound_above_signed(pnl * sign, max_reserve as i128)
    }

    pub fn get_net_borrowing_fee(&self, current_cummulative_borrowing_factor: u128) -> u128 {
        let net_borrowing_factor: u128 =
            current_cummulative_borrowing_factor - self.pre_cummulative_borrowing_factor;
        let open_interest = self.open_interest();
        return apply_precision(net_borrowing_factor, open_interest);
    }

    pub fn get_net_funding_fee(&self, current_cummulative_funding_factor: i128) -> i128 {
        let net_funding_factor: i128 =
            current_cummulative_funding_factor - self.pre_cummulative_funding_factor;
        // if net_funding is positive return
        let sign = if net_funding_factor > 0 { 1 } else { -1 };

        let open_interest = self.open_interest();

        return apply_precision(net_funding_factor.abs() as u128, open_interest) as i128 * sign;
    }

    pub fn open_interest(&self) -> u128 {
        self.debt + self.collateral
    }
}

impl Storable for PositionDetails {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let serialized = bincode::serialize(self).expect("failed to serialize");
        Cow::Owned(serialized)
    }

    /// Converts the element into an owned byte vector.
    ///
    /// This method consumes `self` and avoids cloning when possible.
    fn into_bytes(self) -> Vec<u8> {
        bincode::serialize(&self).expect("failed to serialize")
    }

    /// Converts bytes into an element.
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        bincode::deserialize(bytes.as_ref()).expect("failed to desearalize")
    }

    /// The size bounds of the type.
    const BOUND: Bound = Bound::Bounded {
        max_size: 140,
        is_fixed_size: false,
    };
}

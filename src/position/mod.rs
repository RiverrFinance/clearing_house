use std::borrow::Cow;

use candid::{CandidType, Principal};
use ic_stable_structures::{Storable, storable::Bound};
use serde::{Deserialize, Serialize};

use crate::math::math::{apply_precision, bound_above_signed, bound_below_signed};

/// Position

#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
#[derive(Copy, Clone, Deserialize, CandidType, Serialize)]
pub struct Position {
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

impl Position {
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

    /// ClosePositon With  Net Positive Funding
    ///
    /// and  Close Position with Net Negative Funding ( see below)
    ///
    /// Params
    ///
    /// Free Liquidity - the amount of free liquidity
    /// Net Funding - magnitude of net  funding fee to be received or paid  by position
    /// Net borrowing Fee - net booring fee to be paid by position
    /// Position PNL - the profit or loss of that position
    ///
    /// Returns
    ///
    /// Net free liquidity,
    /// Collateral Out ( this incliudes both the collateral and the profit made) ,
    /// House Debt ,in extreme cases of where a position can not fully pay its funding fee ,
    /// this is repaid by the house and using its free liquidity and if free liquidity is not enough a bad debt occurs
    ///
    ///
    pub fn close_position_with_net_positive_funding(
        &self,
        free_liquidity: u128,
        net_funding_fee_magnitude: i128,
        net_borrowing_fee: u128,
        position_pnl: i128,
    ) -> (u128, u128, u128) {
        let position_pnl_magnitude = position_pnl.abs();
        let open_interest = self.open_interest();

        let mut net_free_liquidity = free_liquidity;

        let net_position_value = bound_below_signed(
            open_interest as i128 + position_pnl + net_funding_fee_magnitude
                - (net_borrowing_fee + self.debt) as i128,
            0,
        );

        if position_pnl.is_negative() {
            // trader losses ,house gains
            let position_value_without_pnl = bound_below_signed(
                open_interest as i128 - (net_borrowing_fee + self.debt) as i128
                    + net_funding_fee_magnitude,
                0,
            );

            let position_loss = position_value_without_pnl.min(position_pnl_magnitude);
            //
            //
            net_free_liquidity += self.max_reserve + position_loss as u128;
        } else {
            // trader gains , house losses
            net_free_liquidity += self.max_reserve - position_pnl_magnitude as u128
        }

        return (net_free_liquidity, net_position_value as u128, 0);

        //net_free_liquidity += reserve
    }

    /// Returns  (net_free_liquidity, funding_paid, collateral_out,bad_debt)
    pub fn close_position_with_net_negative_funding(
        &self,
        free_liquidity: u128,
        net_funding_fee: i128,
        net_borrowing_fee: u128,
        position_pnl: i128,
    ) -> (u128, u128, u128) {
        let position_pnl_magnitude = position_pnl.abs();
        let open_interest = self.open_interest();

        let net_funding_fee_magnitude = net_funding_fee.abs();

        let mut net_free_liquidity = free_liquidity;

        // if pnl is negative
        // the amount to add to free_liquidity is
        // let position_value_without_funding_pay = bound_below_signed(
        //     open_interest as i128 + position_pnl
        //         - (open_interest + net_borrowing_fee + self.debt) as i128,
        //     0,
        // );

        let position_value_without_funding_pay = bound_below_signed(
            open_interest as i128 + position_pnl - (net_borrowing_fee + self.debt) as i128,
            0,
        );

        let mut collateral_out = 0;

        let mut house_debt = 0;

        if position_value_without_funding_pay >= net_funding_fee_magnitude {
            // if position_value after debt is enough to pay funding fee
            // collateral is what is left
            collateral_out =
                (position_value_without_funding_pay - net_funding_fee_magnitude) as u128;

            // net free liquidity is max_reserve - position_pnl
            net_free_liquidity += (self.max_reserve as i128 - position_pnl) as u128
        } else {
            // This block of code  tracks extreme cases like
            // when position funding_fee_can not be paid fully by a position
            if position_pnl.is_negative() {
                let position_value_without_pnl = bound_below_signed(
                    open_interest as i128
                        - (net_borrowing_fee + self.debt) as i128
                        - net_funding_fee_magnitude,
                    0,
                );

                // amount extractable to house
                let position_loss = position_value_without_pnl.min(position_pnl_magnitude);
                //
                //
                net_free_liquidity += self.max_reserve + position_loss as u128;
            } else {
                net_free_liquidity += (self.max_reserve as i128 - position_pnl_magnitude) as u128
            }

            // if funding can not be paid by position,the cost is covered by the huse
            // let delta = i128::max(
            //     net_free_liquidity as i128 + position_value_without_funding_pay // basically free liquidity + anything remainng in position - net funding fee magnitude
            //         - net_funding_fee_magnitude,
            //     0,
            // );

            let delta = net_free_liquidity as i128 + position_value_without_funding_pay // basically free liquidity + anything remainng in position - net funding fee magnitude
                    - net_funding_fee_magnitude;

            if delta >= 0 {
                net_free_liquidity = delta as u128;
            } else {
                net_free_liquidity = 0;
                house_debt = delta.abs() as u128;
                //funding_paid = (delta + net_funding_fee_magnitude) as u128
            }
        }

        return (net_free_liquidity, collateral_out, house_debt);
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

impl Default for Position {
    fn default() -> Self {
        Self {
            owner: Principal::anonymous(),
            long: false,
            collateral: 0,
            max_reserve: 0,
            debt: 0,
            units: 0,
            pre_cummulative_borrowing_factor: 0,
            pre_cummulative_funding_factor: 0,
        }
    }
}

impl Storable for Position {
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

use candid::{CandidType, Principal};
use serde::Deserialize;

use crate::{
    open_position::open_position::_open_position,
    pricing_update_management::price_waiting_operation_trait::PriceWaitingOperation,
};
/// Parameters for opening a new trading position in a market.
///
/// This struct contains all the necessary information to open a leveraged trading position
/// in a specific market. The owner must be the message caller to ensure security.
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
#[derive(CandidType, Clone, Copy, Deserialize)]
pub struct OpenPositionParams {
    /// The principal ID of the position owner.
    /// **IMPORTANT**: This must match the message caller (`msg_caller()`) for security.
    /// The function will fail if this doesn't match the actual caller.
    pub owner: Principal,

    /// The direction of the position.
    /// - `true`: Long position (betting on price increase)
    /// - `false`: Short position (betting on price decrease)
    pub long: bool,

    /// The unique identifier of the target market to open the position in.
    /// Must correspond to an existing market in the clearing house.
    pub market_index: u64,

    /// The collateral amount to be locked for this position.
    /// This should be specified with 20 decimal places precision (e.g., 1000000000000000000000 for 0.1 units).
    /// The user's balance must cover this amount plus the execution fee.
    pub collateral: u128,

    /// The leverage multiplier for the position.
    /// This should be specified with 20 decimal places precision (e.g., 10000000000000000000000 for 10x leverage).
    /// Higher leverage increases both potential profits and risks.
    pub leverage_factor: u128,

    /// The maximum acceptable price for opening the position.
    /// This should be specified with 20 decimal places precision.
    /// If the current market price exceeds this limit, the position opening will fail.
    /// This provides protection against unfavorable price movements.
    pub acceptable_price_limit: u128,

    /// The reserve factor for risk management.
    /// This should be specified with 20 decimal places precision (e.g., 500000000000000000000 for 0.05 units).
    /// This factor is used in risk calculations and position sizing.
    pub reserve_factor: u128,
}

impl PriceWaitingOperation for OpenPositionParams {
    fn execute(&self) {
        _open_position(self);
    }
}

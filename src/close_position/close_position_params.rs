use candid::{CandidType, Principal};
use serde::Deserialize;

use crate::close_position::close_position::_close_position;
use crate::pricing_update_management::price_waiting_operation_trait::{
    PriceWaitingOperation, PriceWaitingOperationTrait,
};

/// Parameters for closing an existing trading position.
///
/// This struct contains all the necessary information to close a trading position
/// in a specific market. The owner must be the message caller to ensure security.
#[derive(CandidType, Deserialize, Copy, Clone)]
pub struct ClosePositionParams {
    /// The unique identifier of the market containing the position to close.
    /// Must correspond to an existing market in the clearing house.
    #[serde(rename = "marketIndex")]
    pub market_index: u64,

    /// The principal ID of the position owner.
    /// **IMPORTANT**: This must match the message caller (`msg_caller()`) for security.
    /// The function will fail if this doesn't match the actual caller.
    pub owner: Principal,

    /// The unique identifier of the position to close.
    /// This must correspond to an existing position owned by the caller.
    /// Position IDs are generated when positions are opened.
    #[serde(rename = "positionId")]
    pub position_id: u64,

    /// The maximum acceptable price for closing the position.
    /// This should be specified with 20 decimal places precision.
    /// If the current market price exceeds this limit, the position closure will fail.
    /// This provides protection against unfavorable price movements during closure.
    #[serde(rename = "acceptablePriceLimit")]
    pub acceptable_price_limit: u128,
}

impl PriceWaitingOperationTrait for ClosePositionParams {
    fn execute(&self) {
        _close_position(&self);
    }
}

impl From<ClosePositionParams> for PriceWaitingOperation {
    fn from(params: ClosePositionParams) -> Self {
        PriceWaitingOperation::ClosePosition(params)
    }
}

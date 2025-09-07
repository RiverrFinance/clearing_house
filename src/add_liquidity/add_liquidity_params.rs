use candid::{CandidType, Principal};
use serde::Deserialize;

use crate::add_liquidity::add_liquidity::_add_liquidity;
use crate::pricing_update_management::price_waiting_operation_trait::PriceWaitingOperation;

/// Parameters for adding liquidity to a market.
///
/// This struct contains all the necessary information to add liquidity to a specific
/// market in the clearing house. The depositor must be the message caller to ensure
/// security and prevent unauthorized operations.
#[derive(CandidType, Deserialize, Copy, Clone)]
pub struct AddLiquidityToMarketParams {
    /// The unique identifier of the target market to add liquidity to.
    /// Must correspond to an existing market in the clearing house.
    pub market_index: u64,

    /// The principal ID of the user adding liquidity.
    /// **IMPORTANT**: This must match the message caller (`msg_caller()`) for security.
    /// The function will fail if this doesn't match the actual caller.
    pub depositor: Principal,

    /// The amount of base asset to deposit into the market's liquidity pool.
    /// This should be specified with 20 decimal places precision (e.g., 10000000000000000000000 for 1.0 unit).
    /// The user's balance must cover this amount plus the execution fee.
    pub amount: u128,

    /// The minimum amount of liquidity shares expected in return.
    /// This provides slippage protection - if the actual shares received would be
    /// less than this amount, the transaction will fail.
    /// Should be calculated based on current market conditions and acceptable slippage.
    /// Also uses 20 decimal places precision.
    pub min_amount_out: u128,
}

impl PriceWaitingOperation for AddLiquidityToMarketParams {
    fn execute(&self) {
        _add_liquidity(&self);
    }
}

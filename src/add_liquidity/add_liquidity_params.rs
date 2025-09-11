use candid::{CandidType, Principal};
use serde::Deserialize;

use crate::add_liquidity::add_liquidity::_add_liquidity;
use crate::market::functions::add_liquidity_to_market::AddLiquidityToMarketParams;
use crate::pricing_update_management::price_waiting_operation_trait::{
    PriceWaitingOperation, PriceWaitingOperationTrait,
};

/// Parameters for adding liquidity to a market.
///
/// This struct contains all the necessary information to add liquidity to a specific
/// market in the clearing house. The depositor must be the message caller to ensure
/// security and prevent unauthorized operations.
#[derive(CandidType, Deserialize, Copy, Clone)]
pub struct AddLiquidityParams {
    /// The unique identifier of the target market to add liquidity to.
    /// Must correspond to an existing market in the clearing house.
    #[serde(rename = "marketIndex")]
    pub market_index: u64,

    /// The principal ID of the user adding liquidity.
    /// **IMPORTANT**: This must match the message caller (`msg_caller()`) for security.
    /// The function will fail if this doesn't match the actual caller.
    pub depositor: Principal,

    /// The quote asset amount to deposit into the market's liquidity pool.
    /// Uses 20-decimal precision (e.g., 10000000000000000000000 for 1.0 quote unit).
    /// The user's balance must cover this amount plus the execution fee (also in quote asset).
    pub amount: u128,

    /// The minimum amount of liquidity shares expected in return.
    /// Shares are distinct units from the quote asset and also use 20-decimal precision.
    /// Acts as slippage protection: if actual shares would be lower, the transaction fails.
    #[serde(rename = "minAmountOut")]
    pub min_amount_out: u128,
}

impl PriceWaitingOperationTrait for AddLiquidityParams {
    fn execute(&self) {
        _add_liquidity(&self);
    }
}

impl Into<AddLiquidityToMarketParams> for AddLiquidityParams {
    fn into(self) -> AddLiquidityToMarketParams {
        let Self {
            min_amount_out,
            amount,
            ..
        } = self;
        AddLiquidityToMarketParams {
            min_amount_out,
            amount,
        }
    }
}

impl From<AddLiquidityParams> for PriceWaitingOperation {
    fn from(params: AddLiquidityParams) -> Self {
        PriceWaitingOperation::AddLiquidity(params)
    }
}

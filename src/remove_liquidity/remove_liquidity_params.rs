use candid::{CandidType, Principal};
use serde::Deserialize;

use crate::{
    market::functions::remove_liquidity::RemoveLiquidityFromMarketParams,
    pricing_update_management::price_waiting_operation_trait::{
        PriceWaitingOperation, PriceWaitingOperationTrait,
    },
    remove_liquidity::remove_liquidity::_remove_liquidity,
};

/// Parameters for removing liquidity from a market.
///
/// This struct contains all the necessary information to remove liquidity shares from a
/// specific market in the clearing house. The owner must be the message caller to ensure security.
#[derive(CandidType, Deserialize, Copy, Clone)]
pub struct RemoveLiquidityParams {
    /// The unique identifier of the target market to remove liquidity from.
    /// Must correspond to an existing market in the clearing house.
    pub market_index: u64,

    /// The principal ID of the liquidity provider.
    /// **IMPORTANT**: This must match the message caller (`msg_caller()`) for security.
    /// The function will fail if this doesn't match the actual caller.
    pub owner: Principal,

    /// The amount of liquidity shares to remove from the market.
    /// This should be specified with 20 decimal places precision (e.g., 1000000000000000000000 for 0.1 units).
    /// The user must have sufficient liquidity shares in the specified market.
    pub amount_in: u128,

    /// The minimum amount of quote asset expected in return.
    /// Uses 20-decimal precision (e.g., 950000000000000000000 for 0.095 quote units).
    /// This provides slippage protection - if the actual assets received would be
    /// less than this amount, the transaction will fail.
    /// Should be calculated based on current market conditions and acceptable slippage.
    pub min_amount_out: u128,
}

impl PriceWaitingOperationTrait for RemoveLiquidityParams {
    fn execute(&self) {
        _remove_liquidity(self);
    }
}

impl From<RemoveLiquidityParams> for PriceWaitingOperation {
    fn from(params: RemoveLiquidityParams) -> Self {
        PriceWaitingOperation::RemoveLiquidity(params)
    }
}

impl Into<RemoveLiquidityFromMarketParams> for RemoveLiquidityParams {
    fn into(self) -> RemoveLiquidityFromMarketParams {
        let Self {
            min_amount_out,
            amount_in,
            ..
        } = self;
        RemoveLiquidityFromMarketParams {
            min_amount_out,
            amount_in,
        }
    }
}

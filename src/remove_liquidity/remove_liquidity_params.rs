use candid::CandidType;
use serde::Deserialize;

use crate::pricing_update_management::price_waiting_operation_arg_variants::MarketLiquidityOperationParams;

#[derive(CandidType, Deserialize, Copy, Clone)]
pub struct RemoveLiquidityFromMarketParams {
    pub amount_in: u128,
    pub min_amount_out: u128,
}

impl Into<MarketLiquidityOperationParams> for RemoveLiquidityFromMarketParams {
    fn into(self) -> MarketLiquidityOperationParams {
        let Self {
            amount_in,
            min_amount_out,
        } = self;

        MarketLiquidityOperationParams {
            amount_in,
            min_amount_out,
        }
    }
}

impl From<MarketLiquidityOperationParams> for RemoveLiquidityFromMarketParams {
    fn from(params: MarketLiquidityOperationParams) -> Self {
        Self {
            amount_in: params.amount_in,
            min_amount_out: params.min_amount_out,
        }
    }
}

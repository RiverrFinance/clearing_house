use candid::CandidType;
use serde::Deserialize;

use crate::pricing_update_management::price_waiting_operation_arg_variants::MarketLiquidityOperationParams;

#[derive(CandidType, Deserialize, Copy, Clone)]
pub struct AddLiquidityToMarketParams {
    pub amount: u128,
    pub min_amount_out: u128,
}

impl Into<MarketLiquidityOperationParams> for AddLiquidityToMarketParams {
    fn into(self) -> MarketLiquidityOperationParams {
        let Self {
            amount,
            min_amount_out,
        } = self;

        MarketLiquidityOperationParams {
            amount_in: amount,
            min_amount_out,
        }
    }
}

impl From<MarketLiquidityOperationParams> for AddLiquidityToMarketParams {
    fn from(params: MarketLiquidityOperationParams) -> Self {
        Self {
            amount: params.amount_in,
            min_amount_out: params.min_amount_out,
        }
    }
}

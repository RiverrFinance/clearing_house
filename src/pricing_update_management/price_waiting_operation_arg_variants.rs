use candid::Principal;

use crate::{
    close_position::close_position_params::ClosePositionParams,
    open_position::open_position_params::OpenPositionParams,
};

#[derive(Clone, Copy)]
pub enum PriceWaitingOperation {
    ClosePositionOp {
        owner: Principal,
        params: ClosePositionParams,
    },
    OpenPositionOp(OpenPositionParams),
    MarketLiquidityOp {
        depositor: Principal,
        adding: bool,
        params: MarketLiquidityOperationParams,
    },
    CollectBorrowingFeesOp,
}

#[derive(Clone, Copy)]
pub struct MarketLiquidityOperationParams {
    pub amount_in: u128,
    pub min_amount_out: u128,
}

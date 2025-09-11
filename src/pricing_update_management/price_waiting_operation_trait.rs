use crate::{
    add_liquidity::add_liquidity_params::AddLiquidityParams,
    admin_roles::collect_borrowing_fees::CollectBorrowFeesParams,
    close_position::close_position_params::ClosePositionParams,
    open_position::open_position_params::OpenPositionParams,
    remove_liquidity::remove_liquidity_params::RemoveLiquidityParams,
};

/// A trait for all price waiting operations to enable a priority list for maximum execution
pub trait PriceWaitingOperationTrait {
    /// excutes the paritucular operation on the market
    fn execute(&self);

    //  fn executor(&self) -> Principal;
}

pub enum PriceWaitingOperation {
    OpenPosition(OpenPositionParams),
    ClosePosition(ClosePositionParams),
    AddLiquidity(AddLiquidityParams),
    RemoveLiquidity(RemoveLiquidityParams),
    CollectBorrowFees(CollectBorrowFeesParams),
}

impl PriceWaitingOperationTrait for PriceWaitingOperation {
    fn execute(&self) {
        match self {
            PriceWaitingOperation::OpenPosition(params) => params.execute(),
            PriceWaitingOperation::ClosePosition(params) => params.execute(),
            PriceWaitingOperation::AddLiquidity(params) => params.execute(),
            PriceWaitingOperation::RemoveLiquidity(params) => params.execute(),
            PriceWaitingOperation::CollectBorrowFees(params) => params.execute(),
        }
    }
}

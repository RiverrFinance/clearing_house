use std::collections::VecDeque;
use std::time::Duration;

use crate::add_liquidity::add_liquidity::_add_liquidity;
use crate::add_liquidity::add_liquidity_params::AddLiquidityToMarketParams;
use crate::close_position::close_position::_close_position;
use crate::open_position::open_position::_open_position;
use crate::pricing_update_management::price_waiting_operation_arg_variants::PriceWaitingOperation;
use crate::remove_liquidity::remove_liquidity::_remove_liquidity;
use crate::remove_liquidity::remove_liquidity_params::RemoveLiquidityFromMarketParams;
use crate::stable_memory::MARKET_PRICE_WAITING_OPERATION;

pub fn put_price_waiting_operation(
    market_index: u64,
    operation: PriceWaitingOperation,
    push_back: bool,
) {
    MARKET_PRICE_WAITING_OPERATION.with_borrow_mut(|reference| {
        let value = reference.remove(&market_index);

        let mut operations_vec: VecDeque<PriceWaitingOperation> = VecDeque::new();
        if value.is_some() {
            let (timer_id, operations) = value.unwrap();
            ic_cdk_timers::clear_timer(timer_id.clone());
            operations_vec = operations
        };

        if push_back {
            operations_vec.push_back(operation);
        } else {
            operations_vec.push_front(operation);
        }
        let new_timer = ic_cdk_timers::set_timer(Duration::from_secs(4), move || {
            ic_cdk::futures::spawn(schedule_execution_of_price_waiting_operations(market_index));
        });
        reference.insert(market_index, (new_timer, operations_vec));

        return;
    });
}

pub async fn schedule_execution_of_price_waiting_operations(market_index: u64) {
    // update_price here
    let (_, operations) = MARKET_PRICE_WAITING_OPERATION.with_borrow_mut(|reference| {
        reference
            .remove(&market_index)
            .expect("Market was removed before oepration started")
    });

    let mut index = 0;

    while index < operations.len() {
        let op = operations[index];

        match op {
            PriceWaitingOperation::ClosePositionOp { owner, params } => {
                _close_position(owner, params);
            }
            PriceWaitingOperation::OpenPositionOp(params) => {
                _open_position(params);
            }
            PriceWaitingOperation::MarketLiquidityOp {
                depositor,
                adding,
                params,
            } => {
                if adding {
                    // refatcor
                    _add_liquidity(
                        market_index,
                        depositor,
                        AddLiquidityToMarketParams::from(params),
                    );
                } else {
                    _remove_liquidity(
                        market_index,
                        depositor,
                        RemoveLiquidityFromMarketParams::from(params),
                    );
                }
            }
            PriceWaitingOperation::CollectBorrowingFeesOp => {}
        }

        index += 1
    }
}

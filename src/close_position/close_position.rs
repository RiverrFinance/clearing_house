use crate::close_position::close_position_result::ClosePositionResult;
use crate::pricing_update_management::price_waiting_operation_arg_variants::PriceWaitingOperation;
use crate::pricing_update_management::price_waiting_operation_utils::put_price_waiting_operation;
use crate::stable_memory::MARKETS;
use crate::user::user_query::{get_user_position_details, try_get_user_position_details};
use crate::{
    close_position::close_position_params::ClosePositionParams,
    user::balance_utils::update_user_balance,
};
use candid::Principal;
use ic_cdk::{api::msg_caller, update};

#[update(name = "closePosition")]
pub fn close_position(params: ClosePositionParams) -> ClosePositionResult {
    let owner = msg_caller();

    let result = _close_position(owner, params);
    if let ClosePositionResult::Waiting = result {
        let (market_index, _) = get_user_position_details(owner, params.position_id);
        put_price_waiting_operation(
            market_index,
            PriceWaitingOperation::ClosePositionOp { owner, params },
            false,
        );
    };

    return result;
}

pub fn _close_position(owner: Principal, params: ClosePositionParams) -> ClosePositionResult {
    MARKETS.with_borrow_mut(|reference| {
        let Some((market_index, position)) =
            try_get_user_position_details(owner, params.position_id)
        else {
            return ClosePositionResult::Failed;
        };

        let mut market = reference.get(market_index).expect("Market does not exist");

        let result = market.close_position(position, params.acceptable_price_limit);

        if let ClosePositionResult::Settled { returns } = result {
            update_user_balance(position.owner, returns, true);
            reference.set(market_index, &market);
        }

        return result;
    })
}

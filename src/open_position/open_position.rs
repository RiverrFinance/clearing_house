use crate::house_settings::get_execution_fee;
use crate::market::functions::open_position_in_market::{
    FailureReason, OpenPositioninMarketResult,
};
use crate::open_position::open_position_params::OpenPositionParams;
use crate::pricing_update_management::price_waiting_operation_arg_variants::PriceWaitingOperation;
use crate::pricing_update_management::price_waiting_operation_utils::{
    is_within_price_update_interval, put_price_waiting_operation,
};
use crate::stable_memory::MARKETS_WITH_LAST_PRICE_UPDATE_TIME;
use crate::user::balance_utils::{get_user_balance, set_user_balance};
use crate::user::position_util::put_user_position_detail;

use ic_cdk::api::time;
use ic_cdk::update;

#[update]
pub fn open_position(params: OpenPositionParams) -> OpenPositioninMarketResult {
    let result = _open_position(params);

    if let OpenPositioninMarketResult::Waiting { params } = result {
        put_price_waiting_operation(
            params.market_index,
            PriceWaitingOperation::OpenPositionOp(params),
            true,
        );
    };

    return result;
}

pub fn _open_position(params: OpenPositionParams) -> OpenPositioninMarketResult {
    MARKETS_WITH_LAST_PRICE_UPDATE_TIME.with_borrow_mut(|reference| {
        let (mut market, last_price_update_time) = reference
            .get(params.market_index)
            .expect("Market does not exist");

        let trader = params.owner;
        let trader_balance = get_user_balance(trader);

        if trader_balance < params.collateral {
            return OpenPositioninMarketResult::Failed {
                reason: FailureReason::InsufficientBalance,
            };
        }

        if is_within_price_update_interval(last_price_update_time) == false {
            let execution_fee = get_execution_fee();
            assert!(
                trader_balance - params.collateral >= execution_fee,
                "Insufficient balance to pay for execution fee "
            );
            // reduce user balance
            set_user_balance(trader, trader_balance - execution_fee);
            return OpenPositioninMarketResult::Waiting { params };
        }

        let result = market.open_position_in_market(params);

        // if it was settled we need to update the user position and balance
        if let OpenPositioninMarketResult::Settled { position } = result {
            set_user_balance(trader, trader_balance - params.collateral);

            let position_id = time();
            put_user_position_detail(trader, params.market_index, position_id, position);

            reference.set(params.market_index, &(market, last_price_update_time));
        };

        return result;
    })
}

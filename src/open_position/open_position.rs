use crate::market::functions::open_position_in_market::{
    FailureReason, OpenPositioninMarketResult,
};
use crate::open_position::open_position_params::OpenPositionParams;
// use crate::pricing_update_management::price_waiting_operation_arg_variants::PriceWaitingOperation;
// use crate::pricing_update_management::price_waiting_operation_utils::put_price_waiting_operation;
use crate::stable_memory::MARKETS;
use crate::user::balance_utils::{get_user_balance, set_user_balance};
use crate::user::position_util::put_user_position_detail;

use ic_cdk::api::time;
use ic_cdk::update;

#[update]
pub async fn open_position(params: OpenPositionParams) -> OpenPositioninMarketResult {
    let result = _open_position(params).await;

    // if let OpenPositioninMarketResult::Waiting { params } = result {
    //     put_price_waiting_operation(
    //         params.market_index,
    //         PriceWaitingOperation::OpenPositionOp(params),
    //         true,
    //     );
    // };

    return result;
}

pub async fn _open_position(params: OpenPositionParams) -> OpenPositioninMarketResult {
    let trader = params.owner;
    let trader_balance = get_user_balance(trader);

    if trader_balance < params.collateral {
        return OpenPositioninMarketResult::Failed {
            reason: FailureReason::InsufficientBalance,
        };
    }
    // check if market exists
    let Some(mut market) = MARKETS.with_borrow_mut(|reference| reference.get(params.market_index))
    else {
        return OpenPositioninMarketResult::Failed {
            reason: FailureReason::Other,
        };
    };

    let result = market.open_position_in_market(params).await;

    // if it was settled we need to update the user position and balance
    if let OpenPositioninMarketResult::Settled { position } = result {
        // reduce user balance
        set_user_balance(trader, trader_balance - params.collateral);

        let position_id = time();
        put_user_position_detail(trader, params.market_index, position_id, position);

        MARKETS.with_borrow_mut(|reference| reference.set(params.market_index, &market));
    };

    return result;
}

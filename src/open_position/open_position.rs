use crate::market::functions::open_position_in_market::OpenPositioninMarketResult;
use crate::open_position::open_position_params::OpenPositionParams;
use crate::pricing_update_management::price_waiting_operation_arg_variants::PriceWaitingOperation;
use crate::pricing_update_management::price_waiting_operation_utils::put_price_waiting_operation;
use crate::stable_memory::MARKETS;
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
    MARKETS.with_borrow_mut(|reference| {
        let mut market = reference
            .get(params.market_index)
            .expect("Market with that indexnt found");

        let trader = params.owner;
        let trader_balance = get_user_balance(trader);

        if trader_balance < params.collateral {
            return OpenPositioninMarketResult::Failed;
        }

        let result = market.open_position_in_market(params);

        // if it was settled we need to update the user position and balance
        if let OpenPositioninMarketResult::Settled { position } = result {
            // reduce user balance
            set_user_balance(trader, trader_balance - params.collateral);

            let position_id = time();
            put_user_position_detail(trader, params.market_index, position_id, position);

            reference.set(params.market_index, &market);
        };

        return result;
    })
}

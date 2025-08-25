use candid::Principal;
use ic_cdk::{api::msg_caller, update};

use crate::{
    market::market_details::LiquidityOperationResult,
    pricing_update_management::{
        price_waiting_operation_arg_variants::PriceWaitingOperation,
        price_waiting_operation_utils::{
            is_within_price_update_interval, put_price_waiting_operation,
        },
    },
    remove_liquidity::remove_liquidity_params::RemoveLiquidityFromMarketParams,
    stable_memory::MARKETS_WITH_LAST_PRICE_UPDATE_TIME,
    user::balance_utils::{
        get_user_market_liquidity_shares, set_user_market_liquidity_shares, update_user_balance,
    },
};

#[update]
pub async fn remove_liquidity(
    market_index: u64,
    params: RemoveLiquidityFromMarketParams,
) -> LiquidityOperationResult {
    let depositor = msg_caller();

    let result = _remove_liquidity(market_index, depositor, params);

    if let LiquidityOperationResult::Waiting = result {
        put_price_waiting_operation(
            market_index,
            PriceWaitingOperation::MarketLiquidityOp {
                depositor,
                adding: false,
                params: params.into(),
            },
            true,
        );
    }

    return result;
}

pub fn _remove_liquidity(
    market_index: u64,
    depositor: Principal,
    params: RemoveLiquidityFromMarketParams,
) -> LiquidityOperationResult {
    let user_shares_balance = get_user_market_liquidity_shares(depositor, market_index);

    if user_shares_balance < params.amount_in {
        return LiquidityOperationResult::Failed;
    };

    MARKETS_WITH_LAST_PRICE_UPDATE_TIME.with_borrow_mut(|reference| {
        let (mut market, last_time_updated) = reference.get(market_index).unwrap();

        // check if price update interval is within the allowed interval
        if is_within_price_update_interval(last_time_updated) == false {
            return LiquidityOperationResult::Waiting;
        }

        let result = market.remove_liquidity_from_market(params);

        if let LiquidityOperationResult::Settled { amount_out } = result {
            set_user_market_liquidity_shares(
                depositor,
                market_index,
                user_shares_balance - params.amount_in,
            );

            update_user_balance(depositor, amount_out, true);

            reference.set(market_index, &(market, last_time_updated));
        }
        return result;
    })
}

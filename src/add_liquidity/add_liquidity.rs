use candid::Principal;
use ic_cdk::{api::msg_caller, update};

use crate::add_liquidity::add_liquidity_params::AddLiquidityToMarketParams;
use crate::house_settings::get_execution_fee;
//use crate::house_settings::get_house_asset_ledger;
use crate::market::market_details::LiquidityOperationResult;
use crate::pricing_update_management::price_waiting_operation_arg_variants::PriceWaitingOperation;
use crate::pricing_update_management::price_waiting_operation_utils::{
    is_within_price_update_interval, put_price_waiting_operation,
};
use crate::stable_memory::MARKETS_WITH_LAST_PRICE_UPDATE_TIME;
use crate::user::balance_utils::{
    get_user_balance, set_user_balance, update_user_market_liquidity_shares,
};

#[update(name = "addLiquidity")]
pub fn add_liquidity(
    market_index: u64,
    params: AddLiquidityToMarketParams,
) -> LiquidityOperationResult {
    let depositor = msg_caller();

    let result = _add_liquidity(market_index, depositor, params);

    if let LiquidityOperationResult::Waiting = result {
        put_price_waiting_operation(
            market_index,
            PriceWaitingOperation::MarketLiquidityOp {
                depositor,
                adding: true,
                params: params.into(),
            },
            false,
        );
    }

    return result;
}

pub fn _add_liquidity(
    market_index: u64,
    depositor: Principal,
    params: AddLiquidityToMarketParams,
) -> LiquidityOperationResult {
    let user_balance = get_user_balance(depositor);

    let execution_fee = get_execution_fee();

    if user_balance < params.amount + execution_fee {
        return LiquidityOperationResult::Failed;
    }

    MARKETS_WITH_LAST_PRICE_UPDATE_TIME.with_borrow_mut(|reference| {
        let (mut market, last_price_update_time) = reference.get(market_index).unwrap();

        // if price check duration has been exhausted
        if is_within_price_update_interval(last_price_update_time) == false {
            // remove execution order first
            set_user_balance(depositor, user_balance - execution_fee);
            return LiquidityOperationResult::Waiting;
        }

        let result = market.add_liquidity_to_market(params);

        if let LiquidityOperationResult::Settled { amount_out } = result {
            set_user_balance(depositor, user_balance - params.amount);

            // let markets_tokens_ledger = get_house_asset_ledger();

            update_user_market_liquidity_shares(depositor, market_index, amount_out, true);

            reference.set(market_index, &(market, last_price_update_time));
        }

        return result;
    })
}

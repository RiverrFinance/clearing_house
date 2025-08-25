use candid::Principal;
use ic_cdk::{api::msg_caller, update};

use crate::add_liquidity::add_liquidity_params::AddLiquidityToMarketParams;
//use crate::house_settings::get_house_asset_ledger;
use crate::market::market_details::LiquidityOperationResult;
use crate::pricing_update_management::price_waiting_operation_arg_variants::PriceWaitingOperation;
use crate::pricing_update_management::price_waiting_operation_utils::put_price_waiting_operation;
use crate::stable_memory::MARKETS;
use crate::user::balance_utils::{
    get_user_balance, set_user_balance, update_user_market_liquidity_shares,
};

#[update(name = "addLiquidity")]
pub async fn add_liquidity(market_index: u64, params: AddLiquidityToMarketParams) {
    let depositor = msg_caller();

    let result = _add_liquidity(market_index, depositor, params).await;

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
}

pub async fn _add_liquidity(
    market_index: u64,
    depositor: Principal,
    params: AddLiquidityToMarketParams,
) -> LiquidityOperationResult {
    let user_balance = get_user_balance(depositor);

    if user_balance < params.amount {
        return LiquidityOperationResult::Failed;
    }

    let mut market = MARKETS.with_borrow_mut(|reference| reference.get(market_index).unwrap());

    let result = market.add_liquidity_to_market(params).await;

    if let LiquidityOperationResult::Settled { amount_out } = result {
        set_user_balance(depositor, user_balance - params.amount);

        // let markets_tokens_ledger = get_house_asset_ledger();

        update_user_market_liquidity_shares(depositor, market_index, amount_out, true);

        // let tx_result = markets_tokens_ledger
        //     ._send_out(amount_out, depositor, Some(market.token_identifier.clone()))
        //     .await;

        // if tx_result == false {
        //     update_user_balance(depositor, params.amount, true);

        //     return LiquidityOperationResult::Failed;
        // }

        MARKETS.with_borrow_mut(|reference| {
            // update market
            reference.set(market_index, &market);
        })
    }

    return result;
}

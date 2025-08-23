use candid::Principal;
use ic_cdk::{api::msg_caller, update};

use crate::{
    market::market_details::LiquidityOperationResult,
    pricing_update_management::{
        price_waiting_operation_arg_variants::PriceWaitingOperation,
        price_waiting_operation_utils::put_price_waiting_operation,
    },
    remove_liquidity::remove_liquidity_params::RemoveLiquidityFromMarketParams,
    stable_memory::MARKETS,
    user::balance_utils::{
        get_user_market_liquidity_shares, update_user_balance, update_user_market_liquidity_shares,
    },
};

#[update]
pub fn remove_liquidity(market_index: u64, params: RemoveLiquidityFromMarketParams) {
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

    let (result, market) = MARKETS.with_borrow_mut(|reference| {
        let mut market = reference.get(market_index).unwrap();

        let result = market.remove_liquidity_from_market(params);

        (result, market)
    });

    if let LiquidityOperationResult::Settled { amount_out } = result {
        update_user_market_liquidity_shares(depositor, market_index, params.amount_in, false);

        // let markets_tokens_ledger = get_house_asset_ledger();

        update_user_balance(depositor, amount_out, true);

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

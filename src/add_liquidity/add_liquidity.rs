use ic_cdk::{api::msg_caller, update};

use crate::add_liquidity::add_liquidity_params::AddLiquidityParams;
use crate::constants::ADD_LIQUIDITY_PRIORITY_INDEX;
use crate::house_settings::{get_execution_fee, update_execution_fees_accumulated};
use crate::market::market_details::LiquidityOperationResult;

use crate::pricing_update_management::price_waiting_operation_trait::PriceWaitingOperation;
use crate::pricing_update_management::price_waiting_operation_utils::put_price_waiting_operation;
use crate::stable_memory::MARKETS_LIST;
use crate::user::balance_utils::{
    get_user_balance, set_user_balance, update_user_market_liquidity_shares,
};

/// Adds liquidity to a specific market in the clearing house.
///
/// This function allows users to deposit assets into a market's liquidity pool,
/// receiving liquidity shares in return. The operation may be executed immediately
/// if price data is current, or queued for later execution if price updates are needed.
///
/// # Parameters
///
/// * `params` - [`AddLiquidityToMarketParams`] containing:
///   - `market_index` (u64): The unique identifier of the target market
///   - `depositor` (Principal): The principal ID of the user adding liquidity
///   - `amount` (u128): Quote asset amount to deposit (20-decimal precision)
///   - `min_amount_out` (u128): Minimum market shares expected in return (slippage protection; shares are distinct from quote asset and use 20-decimal precision)
///
/// # Returns
///
/// Returns [`LiquidityOperationResult`] which can be:
/// - `Settled { amount_out }`: Successfully added liquidity, returns actual shares received
/// - `Waiting`: Operation queued due to stale price data, will execute when price updates
/// - `Failed`: Operation failed due to insufficient balance or invalid parameters
///
/// # Security Notes
///
/// - **Caller Verification**: The `depositor` parameter must match the message caller (`msg_caller()`)
///   to prevent unauthorized operations
/// - **Balance Check**: User must have sufficient balance to cover both the deposit amount
///   and the execution fee
/// - **Execution Fee**: A small execution fee is deducted from the user's balance
///
/// # Price Update Handling
///
/// If the market's price data is stale (beyond the allowed update interval), the operation
/// is queued as a price waiting operation and will be executed automatically when fresh
/// price data becomes available.
///
/// # Units
///
/// - `amount` is in quote asset units (20-decimal precision)
/// - `min_amount_out` is in market share units (20-decimal precision)
///
#[update(name = "addLiquidity")]
pub fn add_liquidity(params: AddLiquidityParams) -> LiquidityOperationResult {
    let depositor = msg_caller();

    assert!(depositor == params.depositor, "Caller is not the depositor");

    let result = _add_liquidity(&params);

    if let LiquidityOperationResult::Waiting { id: _ } = result {
        put_price_waiting_operation(
            params.market_index,
            ADD_LIQUIDITY_PRIORITY_INDEX,
            PriceWaitingOperation::from(params),
        );
    }

    return result;
}

/// Internal implementation of the add liquidity functionality.
///
/// This function performs the core logic for adding liquidity to a market, including:
/// - Balance verification and fee deduction
/// - Price freshness validation
/// - Market state updates
/// - User liquidity share tracking
///
/// # Parameters
///
/// * `params` - Reference to [`AddLiquidityToMarketParams`] containing the operation details
///
/// # Returns
///
/// Returns [`LiquidityOperationResult`] indicating the outcome of the operation.
///
/// # Implementation Details
///
/// The function:
/// 1. Validates user has sufficient balance (amount + execution fee)
/// 2. Checks if market price data is current
/// 3. If price is stale, returns `Waiting` to queue the operation
/// 4. If price is current, executes the liquidity addition
/// 5. Updates user balance and liquidity shares on success
///
/// # Note
///
/// This is an internal function. External callers should use the public `add_liquidity` function
/// which includes proper caller verification and price waiting operation handling.
pub fn _add_liquidity(params: &AddLiquidityParams) -> LiquidityOperationResult {
    let AddLiquidityParams {
        market_index,
        depositor,
        ..
    } = *params;
    let user_balance = get_user_balance(depositor);

    let execution_fee = get_execution_fee();

    if user_balance < params.amount + execution_fee {
        return LiquidityOperationResult::Failed("Insufficient balance".to_string());
    }

    MARKETS_LIST.with_borrow_mut(|reference| {
        let mut market = reference.get(market_index).expect("Market does not exist");

        let result = market.add_liquidity_to_market((*params).into());

        if let LiquidityOperationResult::Settled { amount_out } = result {
            set_user_balance(depositor, user_balance - (params.amount + execution_fee));

            // take excution fee
            update_execution_fees_accumulated(execution_fee, true);

            update_user_market_liquidity_shares(depositor, market_index, amount_out, true);

            reference.set(market_index, &market);
        }

        return result;
    })
}

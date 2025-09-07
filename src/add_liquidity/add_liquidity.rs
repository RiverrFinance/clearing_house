use ic_cdk::{api::msg_caller, update};

use crate::add_liquidity::add_liquidity_params::AddLiquidityToMarketParams;
use crate::constants::ADD_LIQUIDITY_PRIORITY_INDEX;
use crate::house_settings::get_execution_fee;
use crate::market::market_details::LiquidityOperationResult;

use crate::pricing_update_management::price_waiting_operation_utils::{
    is_within_price_update_interval, put_price_waiting_operation,
};
use crate::stable_memory::MARKETS_WITH_LAST_PRICE_UPDATE_TIME;
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
///   - `amount` (u128): The amount of base asset to deposit (with 20 decimal places precision)
///   - `min_amount_out` (u128): Minimum liquidity shares expected in return (slippage protection)
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
/// # Example Usage
///
/// ```rust
/// let params = AddLiquidityToMarketParams {
///     market_index: 0,
///     depositor: msg_caller(),
///     amount: 10000000000000000000000, // 1.0 unit with 20 decimal places precision
///     min_amount_out: 9500000000000000000000, // 0.95 units with 5% slippage tolerance
/// };
///
/// let result = add_liquidity(params);
/// match result {
///     LiquidityOperationResult::Settled { amount_out } => {
///         // Successfully added liquidity, received `amount_out` shares
///     },
///     LiquidityOperationResult::Waiting => {
///         // Operation queued, will execute when price updates
///     },
///     LiquidityOperationResult::Failed => {
///         // Operation failed, check balance and parameters
///     }
/// }
/// ```
#[update(name = "addLiquidity")]
pub fn add_liquidity(params: AddLiquidityToMarketParams) -> LiquidityOperationResult {
    let depositor = msg_caller();

    assert!(depositor == params.depositor, "Caller is not the depositor");

    let result = _add_liquidity(&params);

    if let LiquidityOperationResult::Waiting = result {
        put_price_waiting_operation(
            params.market_index,
            ADD_LIQUIDITY_PRIORITY_INDEX,
            Box::new(params),
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
pub fn _add_liquidity(params: &AddLiquidityToMarketParams) -> LiquidityOperationResult {
    let AddLiquidityToMarketParams {
        market_index,
        depositor,
        ..
    } = *params;
    let user_balance = get_user_balance(depositor);

    let execution_fee = get_execution_fee();

    if user_balance < params.amount + execution_fee {
        return LiquidityOperationResult::Failed;
    }

    MARKETS_WITH_LAST_PRICE_UPDATE_TIME.with_borrow_mut(|reference| {
        let Some((mut market, last_price_update_time)) = reference.get(market_index) else {
            return LiquidityOperationResult::Failed;
        };

        // if price check duration has been exhausted
        if is_within_price_update_interval(last_price_update_time) == false {
            return LiquidityOperationResult::Waiting;
        }

        let result = market.add_liquidity_to_market(*params);

        if let LiquidityOperationResult::Settled { amount_out } = result {
            set_user_balance(depositor, user_balance - (params.amount + execution_fee));

            // let markets_tokens_ledger = get_house_asset_ledger();

            update_user_market_liquidity_shares(depositor, market_index, amount_out, true);

            reference.set(market_index, &(market, last_price_update_time));
        }

        return result;
    })
}

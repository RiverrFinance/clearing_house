use ic_cdk::{api::msg_caller, update};

use crate::{
    constants::REMOVE_LIQUIDITY_PRIORITY_INDEX,
    house_settings::{get_execution_fee, update_execution_fees_accumulated},
    market::market_details::LiquidityOperationResult,
    pricing_update_management::{
        price_waiting_operation_trait::PriceWaitingOperation,
        price_waiting_operation_utils::put_price_waiting_operation,
    },
    remove_liquidity::remove_liquidity_params::RemoveLiquidityParams,
    stable_memory::MARKETS_LIST,
    user::balance_utils::{
        get_user_market_liquidity_shares, set_user_market_liquidity_shares, update_user_balance,
    },
};

/// Removes liquidity from a specific market in the clearing house.
///
/// This function allows users to withdraw their liquidity shares from a market's
/// liquidity pool, receiving the underlying assets in return. The operation may be
/// executed immediately if price data is current, or queued for later execution if
/// price updates are needed.
///
/// # Parameters
///
/// * `params` - [`RemoveLiquidityFromMarketParams`] containing:
///   - `market_index` (u64): The unique identifier of the target market
///   - `owner` (Principal): The principal ID of the liquidity provider
///   - `amount_in` (u128): Liquidity shares to remove (20-decimal precision; shares are distinct from quote asset)
///   - `min_amount_out` (u128): Minimum quote asset expected in return (slippage protection; 20-decimal precision)
///
/// # Returns
///
/// Returns [`LiquidityOperationResult`] which can be:
/// - `Settled { amount_out }`: Successfully removed liquidity, returns actual assets received
/// - `Waiting`: Operation queued due to stale price data, will execute when price updates
/// - `Failed`: Operation failed due to insufficient shares or invalid parameters
///
/// # Security Notes
///
/// - **Caller Verification**: The `owner` parameter must match the message caller (`msg_caller()`)
///   to prevent unauthorized liquidity removal
/// - **Share Balance Check**: User must have sufficient liquidity shares in the specified market
/// - **Balance Update**: Received assets are added to the user's balance upon successful removal
///
/// # Price Update Handling
///
/// If the market's price data is stale (beyond the allowed update interval), the operation
/// is queued as a price waiting operation and will be executed automatically when fresh
/// price data becomes available.
///
/// # Units
///
/// - `amount_in` is specified in market share units (20-decimal precision)
/// - `min_amount_out` is specified in quote asset units (20-decimal precision)
///
/// # Example Usage
///
/// ```rust
/// let params = RemoveLiquidityFromMarketParams {
///     market_index: 0,
///     owner: msg_caller(),
///     amount_in: 1000000000000000000000, // 0.1 units of liquidity shares with 20 decimal places
///     min_amount_out: 950000000000000000000, // 0.095 units minimum assets with 5% slippage tolerance
/// };
///
/// let result = remove_liquidity(params).await;
/// match result {
///     LiquidityOperationResult::Settled { amount_out } => {
///         // Successfully removed liquidity, received `amount_out` assets
///     },
///     LiquidityOperationResult::Waiting => {
///         // Operation queued, will execute when price updates
///     },
///     LiquidityOperationResult::Failed => {
///         // Operation failed, check liquidity shares and parameters
///     }
/// }
/// ```
#[update(name = "removeLiquidity")]
pub async fn remove_liquidity(params: RemoveLiquidityParams) -> LiquidityOperationResult {
    let owner = msg_caller();

    assert!(
        owner == params.owner,
        "Caller is not the owner of the liquidity"
    );

    let result = _remove_liquidity(&params);

    if let LiquidityOperationResult::Waiting { id: _ } = result {
        let tx_id = put_price_waiting_operation(
            params.market_index,
            REMOVE_LIQUIDITY_PRIORITY_INDEX,
            PriceWaitingOperation::from(params),
        ) as u64;

        return LiquidityOperationResult::Waiting {
            id: Some((params.market_index, REMOVE_LIQUIDITY_PRIORITY_INDEX, tx_id)),
        };
    }

    return result;
}

/// Internal implementation of the remove liquidity functionality.
///
/// This function performs the core logic for removing liquidity from a market, including:
/// - Liquidity share balance verification
/// - Price freshness validation
/// - Market state updates
/// - Asset distribution and balance updates
///
/// # Parameters
///
/// * `params` - Reference to [`RemoveLiquidityFromMarketParams`] containing the operation details
///
/// # Returns
///
/// Returns [`LiquidityOperationResult`] indicating the outcome of the operation.
///
/// # Implementation Details
///
/// The function:
/// 1. Validates user has sufficient liquidity shares in the specified market
/// 2. Checks if market price data is current
/// 3. If price is stale, returns `Waiting` to queue the operation
/// 4. If price is current, executes the liquidity removal
/// 5. Updates user liquidity shares and balance on success
///
/// # Note
///
/// This is an internal function. External callers should use the public `remove_liquidity` function
/// which includes proper caller verification and price waiting operation handling.
pub fn _remove_liquidity(params: &RemoveLiquidityParams) -> LiquidityOperationResult {
    let RemoveLiquidityParams {
        amount_in,
        market_index,
        owner,
        ..
    } = *params;

    let user_shares_balance = get_user_market_liquidity_shares(owner, market_index);

    if user_shares_balance < amount_in {
        return LiquidityOperationResult::Failed(
            "User shares balance is less than amount in".to_string(),
        );
    };

    MARKETS_LIST.with_borrow_mut(|reference| {
        let mut market = reference.get(market_index).unwrap();

        let result = market.remove_liquidity_from_market((*params).into());

        if let LiquidityOperationResult::Settled { amount_out } = result {
            set_user_market_liquidity_shares(owner, market_index, user_shares_balance - amount_in);

            let execution_fee = get_execution_fee();

            let execution_fee_gotten = execution_fee.min(amount_out);

            update_execution_fees_accumulated(execution_fee_gotten, true);

            update_user_balance(owner, amount_out - execution_fee_gotten, true);
            reference.set(market_index, &market);
        }
        return result;
    })
}

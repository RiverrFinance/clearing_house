use crate::close_position::close_position_result::ClosePositionResult;

use crate::close_position::close_position_params::ClosePositionParams;
use crate::constants::CLOSE_POSITION_PRIORITY_INDEX;
use crate::pricing_update_management::price_waiting_operation_utils::{
    is_within_price_update_interval, put_price_waiting_operation,
};
use crate::stable_memory::MARKETS_WITH_LAST_PRICE_UPDATE_TIME;
use crate::user::balance_utils::update_user_balance;
use crate::user::user_query::get_user_position_details;
use candid::Principal;
use ic_cdk::{api::msg_caller, update};

/// Closes an existing trading position in a specific market.
///
/// This function allows users to close their existing trading positions and receive
/// the settlement amount. The operation may be executed immediately if price data is
/// current, or queued for later execution if price updates are needed.
///
/// # Parameters
///
/// * `params` - [`ClosePositionParams`] containing:
///   - `market_index` (u64): The unique identifier of the market containing the position
///   - `owner` (Principal): The principal ID of the position owner
///   - `position_id` (u64): The unique identifier of the position to close
///   - `acceptable_price_limit` (u128): Maximum acceptable price for closing (with 20 decimal places precision)
///
/// # Returns
///
/// Returns [`ClosePositionResult`] which can be:
/// - `Settled { returns }`: Successfully closed position, returns settlement amount
/// - `Waiting`: Operation queued due to stale price data, will execute when price updates
/// - `Failed`: Operation failed due to invalid position or other errors
///
/// # Security Notes
///
/// - **Caller Verification**: The `owner` parameter must match the message caller (`msg_caller()`)
///   to prevent unauthorized position closure
/// - **Position Ownership**: Only the position owner can close their own positions
/// - **Balance Update**: Settlement amount is added to the user's balance upon successful closure
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
/// let params = ClosePositionParams {
///     market_index: 0,
///     owner: msg_caller(),
///     position_id: 12345,
///     acceptable_price_limit: 2000000000000000000000, // 2.0 units max price with 20 decimal places
/// };
///
/// let result = close_position(params);
/// match result {
///     ClosePositionResult::Settled { returns } => {
///         // Successfully closed position, received settlement amount
///     },
///     ClosePositionResult::Waiting => {
///         // Operation queued, will execute when price updates
///     },
///     ClosePositionResult::Failed => {
///         // Operation failed, check position ID and ownership
///     }
/// }
/// ```
#[update(name = "closePosition")]
pub fn close_position(params: ClosePositionParams) -> ClosePositionResult {
    let owner = msg_caller();

    let result = _close_position(owner, &params);
    if let ClosePositionResult::Waiting = result {
        put_price_waiting_operation(
            params.market_index,
            CLOSE_POSITION_PRIORITY_INDEX,
            Box::new(params),
        );
    };

    return result;
}

/// Internal implementation of the close position functionality.
///
/// This function performs the core logic for closing a trading position, including:
/// - Position ownership verification
/// - Price freshness validation
/// - Market state updates
/// - Settlement calculation and balance updates
///
/// # Parameters
///
/// * `owner` - The principal ID of the position owner (from message caller)
/// * `params` - Reference to [`ClosePositionParams`] containing the closure details
///
/// # Returns
///
/// Returns [`ClosePositionResult`] indicating the outcome of the operation.
///
/// # Implementation Details
///
/// The function:
/// 1. Verifies the caller is the position owner
/// 2. Retrieves the position details from user records
/// 3. Checks if market price data is current
/// 4. If price is stale, returns `Waiting` to queue the operation
/// 5. If price is current, executes the position closure
/// 6. Updates user balance with settlement amount on success
///
/// # Note
///
/// This is an internal function. External callers should use the public `close_position` function
/// which includes proper caller verification and price waiting operation handling.
pub fn _close_position(owner: Principal, params: &ClosePositionParams) -> ClosePositionResult {
    assert!(
        owner == params.owner,
        "Caller is not the owner of the position"
    );

    let (market_index, position) = get_user_position_details(params.owner, params.position_id);
    MARKETS_WITH_LAST_PRICE_UPDATE_TIME.with_borrow_mut(|reference| {
        let (mut market, last_price_update_time) =
            reference.get(market_index).expect("Market does not exist");

        // check timer
        if is_within_price_update_interval(last_price_update_time) == false {
            return ClosePositionResult::Waiting;
        }

        let result = market.close_position_in_market(position, params.acceptable_price_limit);

        if let ClosePositionResult::Settled { returns } = result {
            update_user_balance(position.owner, returns, true);
            reference.set(market_index, &(market, last_price_update_time));
        }

        return result;
    })
}

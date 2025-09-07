use crate::constants::OPEN_POSITION_PRIORITY_INDEX;
use crate::house_settings::get_execution_fee;
use crate::market::functions::open_position_in_market::{
    FailureReason, OpenPositioninMarketResult,
};
use crate::open_position::open_position_params::OpenPositionParams;
use crate::pricing_update_management::price_waiting_operation_utils::{
    is_within_price_update_interval, put_price_waiting_operation,
};
use crate::stable_memory::MARKETS_WITH_LAST_PRICE_UPDATE_TIME;
use crate::user::balance_utils::{get_user_balance, set_user_balance};
use crate::user::position_util::_put_user_position_detail;

use ic_cdk::api::{msg_caller, time};
use ic_cdk::update;

/// Opens a new trading position in a specific market.
///
/// This function allows users to open leveraged trading positions in markets. The operation
/// may be executed immediately if price data is current, or queued for later execution if
/// price updates are needed. The position is created with specified collateral and leverage.
///
/// # Parameters
///
/// * `params` - [`OpenPositionParams`] containing:
///   - `owner` (Principal): The principal ID of the position owner
///   - `long` (bool): True for long position, false for short position
///   - `market_index` (u64): The unique identifier of the target market
///   - `collateral` (u128): The collateral amount (with 20 decimal places precision)
///   - `leverage_factor` (u128): The leverage multiplier (with 20 decimal places precision)
///   - `acceptable_price_limit` (u128): Maximum acceptable price for the position (with 20 decimal places precision)
///   - `reserve_factor` (u128): Reserve factor for risk management (with 20 decimal places precision)
///
/// # Returns
///
/// Returns [`OpenPositioninMarketResult`] which can be:
/// - `Settled { position }`: Successfully opened position, returns position details
/// - `Waiting`: Operation queued due to stale price data, will execute when price updates
/// - `Failed { reason }`: Operation failed with specific reason (InsufficientBalance, PriceLimitExceeded, Other)
///
/// # Security Notes
///
/// - **Caller Verification**: The `owner` parameter must match the message caller (`msg_caller()`)
///   to prevent unauthorized position creation
/// - **Balance Check**: User must have sufficient balance to cover both the collateral amount
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
/// let params = OpenPositionParams {
///     owner: msg_caller(),
///     long: true, // Long position
///     market_index: 0,
///     collateral: 1000000000000000000000, // 0.1 units collateral with 20 decimal places
///     leverage_factor: 10000000000000000000000, // 10x leverage with 20 decimal places
///     acceptable_price_limit: 2000000000000000000000, // 2.0 units max price with 20 decimal places
///     reserve_factor: 500000000000000000000, // 0.05 units reserve with 20 decimal places
/// };
///
/// let result = open_position(params);
/// match result {
///     OpenPositioninMarketResult::Settled { position } => {
///         // Successfully opened position, received position details
///     },
///     OpenPositioninMarketResult::Waiting => {
///         // Operation queued, will execute when price updates
///     },
///     OpenPositioninMarketResult::Failed { reason } => {
///         // Operation failed, check reason and user balance
///     }
/// }
/// ```
#[update(name = "openPosition")]
pub fn open_position(params: OpenPositionParams) -> OpenPositioninMarketResult {
    let caller = msg_caller();
    assert!(
        caller == params.owner,
        "Caller is not the owner of the position"
    );
    let result = _open_position(&params);

    if let OpenPositioninMarketResult::Waiting = result {
        put_price_waiting_operation(
            params.market_index,
            OPEN_POSITION_PRIORITY_INDEX,
            Box::new(params),
        );
    };

    return result;
}

/// Internal implementation of the open position functionality.
///
/// This function performs the core logic for opening a trading position, including:
/// - Balance verification and fee deduction
/// - Price freshness validation
/// - Market state updates
/// - Position creation and tracking
///
/// # Parameters
///
/// * `params` - Reference to [`OpenPositionParams`] containing the position details
///
/// # Returns
///
/// Returns [`OpenPositioninMarketResult`] indicating the outcome of the operation.
///
/// # Implementation Details
///
/// The function:
/// 1. Validates user has sufficient balance (collateral + execution fee)
/// 2. Checks if market price data is current
/// 3. If price is stale, returns `Waiting` to queue the operation
/// 4. If price is current, executes the position opening
/// 5. Updates user balance and creates position record on success
///
/// # Note
///
/// This is an internal function. External callers should use the public `open_position` function
/// which includes proper caller verification and price waiting operation handling.
pub fn _open_position(params: &OpenPositionParams) -> OpenPositioninMarketResult {
    let trader = params.owner;
    let trader_balance = get_user_balance(trader);

    let execution_fee = get_execution_fee();

    if trader_balance < params.collateral + execution_fee {
        return OpenPositioninMarketResult::Failed {
            reason: FailureReason::InsufficientBalance,
        };
    }
    MARKETS_WITH_LAST_PRICE_UPDATE_TIME.with_borrow_mut(|reference| {
        let (mut market, last_price_update_time) = reference
            .get(params.market_index)
            .expect("Market does not exist");

        if is_within_price_update_interval(last_price_update_time) == false {
            return OpenPositioninMarketResult::Waiting;
        }

        let result = market.open_position_in_market(*params);

        // if it was settled we need to update the user position and balance
        if let OpenPositioninMarketResult::Settled { position } = result {
            set_user_balance(trader, trader_balance - (params.collateral + execution_fee));

            let position_id = time();
            _put_user_position_detail(trader, params.market_index, position_id, position);

            reference.set(params.market_index, &(market, last_price_update_time));
        };

        return result;
    })
}

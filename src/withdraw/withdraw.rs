use ic_cdk::{api::msg_caller, update};

use crate::{
    house_settings::get_house_asset_ledger,
    user::balance_utils::{get_user_balance, set_user_balance},
    withdraw::withdraw_params::WithdrawParams,
};

/// Withdraws assets from a user's account to the house asset ledger.
///
/// This function allows users to withdraw assets from their account balance in the
/// clearing house to the house asset ledger. The withdrawal is processed asynchronously
/// and includes automatic refund protection if the ledger transaction fails.
///
/// # Parameters
///
/// * `params` - [`WithdrawParams`] containing:
///   - `amount` (u128): Quote asset amount to withdraw (20-decimal precision)
///
/// # Returns
///
/// Returns `bool` indicating success:
/// - `true`: Withdrawal was successful and assets were sent to the ledger
/// - `false`: Withdrawal failed, user balance was refunded
///
/// # Security Notes
///
/// - **Caller Verification**: The function uses `msg_caller()` to identify the withdrawer
/// - **Balance Check**: User must have sufficient balance to cover the withdrawal amount
/// - **Refund Protection**: If the ledger transaction fails, the user's balance is automatically refunded
/// - **Atomic Operation**: Balance is deducted before sending to ledger, ensuring consistency
///
/// # Process Flow
///
/// 1. Identifies the caller as the withdrawer
/// 2. Validates user has sufficient balance
/// 3. Deducts the amount from user's balance
/// 4. Sends the withdrawal transaction to the house asset ledger
/// 5. If ledger transaction fails, refunds the user's balance
///
/// # Example Usage
///
/// ```rust
/// let params = WithdrawParams {
///     amount: 10000000000000000000000, // 1.0 unit with 20 decimal places precision
/// };
///
/// let success = withdraw_from_account(params).await;
/// if success {
///     // Withdrawal successful, assets sent to ledger
/// } else {
///     // Withdrawal failed, balance was refunded
/// }
/// ```
#[update(name = "withdrawFromAccount")]
pub async fn withdraw_from_account(params: WithdrawParams) -> bool {
    let user = msg_caller();

    let house_asset_ledger = get_house_asset_ledger();

    let user_balance = get_user_balance(user);

    assert!(user_balance > params.amount, "Insufficient balance");
    set_user_balance(user, user_balance - params.amount);
    let tx_result = house_asset_ledger._send_out(params.amount, user).await;
    if tx_result == false {
        //refund
        set_user_balance(user, user_balance + params.amount);
    }

    tx_result
}

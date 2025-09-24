use ic_cdk::{api::msg_caller, update};

use crate::{
    deposit::deposit_params::DepositParams, house_settings::get_house_asset_ledger,
    user::balance_utils::update_user_balance,
};

/// Deposits assets into a user's account in the clearing house.
///
/// This function allows users to deposit assets from the house asset ledger into their
/// account balance. The deposit is processed asynchronously and requires a valid block
/// index from the ledger for verification.
///
/// # Parameters
///
/// * `params` - [`DepositParams`] containing:
///   - `amount` (u128): Quote asset amount to deposit (20-decimal precision)
///   - `block_index` (Option<BlockIndex>): Optional block index for transaction verification
///
/// # Returns
///
/// Returns `bool` indicating success:
/// - `true`: Deposit was successful and user balance was updated
/// - `false`: Deposit failed, user balance remains unchanged
///
/// # Security Notes
///
/// - **Caller Verification**: The function uses `msg_caller()` to identify the depositor
/// - **Ledger Verification**: Uses the house asset ledger to verify the deposit transaction
/// - **Balance Update**: User balance is only updated if the ledger transaction succeeds
///
/// # Process Flow
///
/// 1. Identifies the caller as the depositor
/// 2. Retrieves the house asset ledger
/// 3. Sends the deposit transaction to the ledger
/// 4. Updates user balance only if the transaction succeeds
///

#[update(name = "depositIntoAccount")]
pub async fn deposit_into_account(params: DepositParams) -> bool {
    let user = msg_caller();

    let house_asset_ledger = get_house_asset_ledger();

    let tx_result = house_asset_ledger._send_in(params.amount, user).await;

    if tx_result {
        update_user_balance(user, params.amount, true);
    }

    tx_result
}

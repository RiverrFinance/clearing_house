use candid::{Nat, Principal};
use ic_cdk::call::Call;
use icrc_ledger_types::{
    icrc1::account::Account,
    icrc2::transfer_from::{TransferFromArgs, TransferFromError},
};

/// Transfers ICRC2 tokens from one account to another using the spender's allowance
///
/// # Arguments
/// * `amount` - Amount of tokens to transfer
/// * `ledger_id` - Principal ID of the token's ledger canister
/// * `from_account` - Source account to transfer from
/// * `to_account` - Destination account to transfer to
///
/// # Returns
/// * `bool` - True if transfer succeeded, false otherwise
///
/// # Notes
/// - Uses ICRC2 standard transferFrom call
/// - Requires prior approval/allowance from source account for the None subaccount of the canister
/// - Does not specify fee, memo, timestamp or spender subaccount (all None)
/// - Returns false on any error in the transfer
/// - Handles nested Result types from IC ledger response
pub async fn send_asset_in_asset_icrc(
    amount: u128,
    ledger_id: Principal,
    from_account: Account,
    to_account: Account,
) -> bool {
    let args = TransferFromArgs {
        spender_subaccount: None,
        from: from_account,
        to: to_account,
        amount: Nat::from(amount),
        fee: None,
        memo: None,
        created_at_time: None,
    };

    let tx_result: Result<Nat, TransferFromError>;

    let call = Call::unbounded_wait(ledger_id, "icrc2_transfer_from").with_arg(args);

    if let Ok(result) = call.await {
        tx_result = result.candid().unwrap();
        return tx_result.is_ok();
    } else {
        return false;
    }
}

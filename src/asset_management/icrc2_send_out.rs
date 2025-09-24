use candid::{Nat, Principal};
use ic_cdk::call::Call;
use icrc_ledger_types::{
    icrc1::account::{Account, Subaccount},
    icrc1::transfer::{TransferArg, TransferError},
};
/// Transfers ICRC tokens from the canister to an external account
///
/// # Arguments
/// * `amount` - Amount of tokens to transfer
/// * `ledger_id` - Principal ID of the token's ledger canister
/// * `from_subaccount` - Optional subaccount to transfer from
/// * `to_account` - Destination account details
///
/// # Returns
/// * `bool` - True if transfer succeeded, false otherwise
///
/// # Notes
/// - Uses ICRC1 standard transfer call
/// - Does not specify fee, memo or timestamp (all None)
/// - Returns false on any error in the transfer
/// - Handles nested Result types from IC ledger response

pub async fn send_asset_out_icrc(
    amount: u128,
    ledger_id: Principal,
    from_subaccount: Option<Subaccount>,
    to_account: Account,
) -> bool {
    // Error: Typo in struct name ICRCTransferrgs -> ICRCTransferArgs
    let args = TransferArg {
        amount: Nat::from(amount),
        from_subaccount,
        to: to_account,
        fee: None,
        created_at_time: None,
        memo: None,
    };

    let tx_result: Result<Nat, TransferError>;

    let call = Call::unbounded_wait(ledger_id, "icrc1_transfer").with_arg(args);

    if let Ok(result) = call.await {
        tx_result = result.candid().unwrap();
        return tx_result.is_ok();
    } else {
        return false;
    }
}

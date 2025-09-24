use candid::Principal;
use ic_ledger_types::{
    AccountIdentifier, DEFAULT_FEE, DEFAULT_SUBACCOUNT, Memo, Subaccount as ICSubaccount, Tokens,
    TransferArgs as ICRCTransferArgs, transfer,
};
use icrc_ledger_types::icrc1::account::{Account, Subaccount};

/// Transfers ICP tokens between accounts on the Internet Computer
///
/// # Arguments
/// * `amount` - Amount of ICP tokens to transfer (in e8s)
/// * `ledger_id` - Principal ID of the ICP ledger canister
/// * `from_sub` - Optional subaccount to transfer from
/// * `to_account` - Destination account details including owner and subaccount
///
/// # Returns
/// * `bool` - True if transfer succeeded, false otherwise
///
/// # Notes
/// - Returns early with true if amount is 0
/// - Uses default fee and memo(0) for all transfers
/// - Handles nested Result types from IC ledger response
///
pub async fn send_asset_out_icp(
    amount: u128,
    ledger_id: Principal,
    from_sub: Option<Subaccount>,
    to_account: Account,
) -> bool {
    let tokens = Tokens::from_e8s(amount as u64);
    if tokens < DEFAULT_FEE {
        return false;
    }
    let args = ICRCTransferArgs {
        amount: Tokens::from_e8s(amount as u64),
        memo: Memo(0),
        fee: DEFAULT_FEE,
        from_subaccount: Some(_to_ic_subaccount(from_sub)),
        to: AccountIdentifier::new(&to_account.owner, &_to_ic_subaccount(to_account.subaccount)),
        created_at_time: None,
    };

    match transfer(ledger_id, &args).await {
        Ok(res) => {
            if let Ok(_) = res {
                return true;
            } else {
                return false;
            }
        }
        Err(_) => return false,
    };
}

fn _to_ic_subaccount(sub: Option<Subaccount>) -> ICSubaccount {
    match sub {
        Some(res) => return ICSubaccount(res),
        None => return DEFAULT_SUBACCOUNT,
    }
}

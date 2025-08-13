use candid::{CandidType, Nat, Principal};
use ic_cdk::call::Call;
use ic_ledger_types::{
    AccountIdentifier, BlockIndex, DEFAULT_FEE, DEFAULT_SUBACCOUNT, GetBlocksArgs, Memo,
    QueryBlocksResponse, Subaccount as ICSubaccount, Tokens, TransferArgs as ICRCTransferArgs,
    query_blocks, transfer,
};
use icrc_ledger_types::{
    icrc1::{
        account::{Account, Subaccount},
        transfer::{TransferArg, TransferError},
    },
    icrc2::transfer_from::{TransferFromArgs, TransferFromError},
};
use serde::{Deserialize, Serialize};

use crate::math::math::apply_precision;

type Amount = u128;

#[derive(Deserialize, Serialize, Copy, Clone, CandidType)]
pub enum AssetClass {
    /// The cryptocurrency asset class.
    Cryptocurrency,
    /// The fiat currency asset class.
    FiatCurrency,
}

impl Default for AssetClass {
    fn default() -> Self {
        AssetClass::Cryptocurrency
    }
}

#[derive(Clone, Serialize, Deserialize, Default, CandidType)]
pub struct AssetPricingDetails {
    /// The symbol/code of the asset.
    pub symbol: String,
    /// The asset class.
    pub class: AssetClass,
}

#[derive(Clone, Copy, Serialize, Deserialize, CandidType)]
pub struct OtherDetails {
    pub canister_id: Principal,
    pub decimals: u32,
    pub asset_type: AssetType,
}

impl Default for OtherDetails {
    fn default() -> Self {
        Self {
            canister_id: Principal::anonymous(),
            decimals: 0,
            asset_type: AssetType::ICP,
        }
    }
}
#[derive(Clone, Serialize, Copy, Deserialize, CandidType)]
pub enum AssetType {
    ICP,
    ICRC,
    RASSET,
}

#[derive(Serialize, Deserialize, CandidType, Clone)]

pub struct Asset {
    pub pricing_details: AssetPricingDetails,
    pub other_details: Option<OtherDetails>,
}

impl Default for Asset {
    fn default() -> Self {
        Self {
            pricing_details: AssetPricingDetails::default(),
            other_details: None,
        }
    }
}

impl Asset {
    pub async fn _send_in(&self, amount: u128, from: Principal) {
        let OtherDetails {
            canister_id,
            decimals,
            asset_type,
        } = self.other_details.unwrap();
        let factored_amout = apply_precision(amount, 10u128.pow(decimals));

        match asset_type {
            AssetType::ICP => {}
            AssetType::ICRC => {}
            AssetType::RASSET => {}
        }
    }
}

async fn _verify_send_in(user: Principal, ledger_id: Principal, index: BlockIndex) -> bool {
    let args = GetBlocksArgs {
        start: index,
        length: 1,
    };
    let block = query_blocks(ledger_id, &args).await;
    match block {
        Ok(response) => {
            let block = response.blocks.get(index).or_else(|| return false);
        }
        Err(_) => return false,
    }

    return true;
}

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

async fn move_asset_icp(
    amount: Amount,
    ledger_id: Principal,
    from_sub: Option<Subaccount>,
    to_account: Account,
) -> bool {
    if amount == 0 {
        return true;
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

async fn send_asset_out_icrc(
    amount: Amount,
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
    amount: Amount,
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

fn _to_ic_subaccount(sub: Option<Subaccount>) -> ICSubaccount {
    match sub {
        Some(res) => return ICSubaccount(res),
        None => return DEFAULT_SUBACCOUNT,
    }
}

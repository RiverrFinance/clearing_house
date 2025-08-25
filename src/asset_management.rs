use candid::{CandidType, Nat, Principal};
use ic_cdk::call::Call;
use ic_ledger_types::{
    AccountIdentifier, BlockIndex, DEFAULT_FEE, DEFAULT_SUBACCOUNT, GetBlocksArgs, Memo, Operation,
    Subaccount as ICSubaccount, Tokens, Transaction, TransferArgs as ICRCTransferArgs,
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

type Amount = u128;

#[derive(Serialize, Copy, Clone, Deserialize, CandidType)]
pub enum AssetLedgerType {
    ICP,
    ICRC,
    RASSET,
}

#[derive(Serialize, Deserialize, CandidType, Copy, Clone)]

pub struct AssetLedger {
    pub ledger_id: Principal,
    pub asset_decimals: u32,
    pub ledger_type: AssetLedgerType,
}

impl Default for AssetLedger {
    fn default() -> Self {
        Self {
            ledger_id: Principal::anonymous(),
            asset_decimals: 0,
            ledger_type: AssetLedgerType::ICRC,
        }
    }
}

impl AssetLedger {}

impl AssetLedger {
    pub async fn _send_in(
        &self,
        amount: u128,
        from: Principal,
        block_index: Option<BlockIndex>,
        token_identifier: Option<String>,
    ) -> bool {
        return true;
        // let Self {
        //     ledger_id,
        //     asset_decimals: decimals,
        //     ledger_type: asset_type,
        // } = self;
        // let factored_amout = apply_precision(amount, 10u128.pow(*decimals));

        // match asset_type {
        //     AssetLedgerType::ICRC => {
        //         let result = send_asset_in_asset_icrc(
        //             factored_amout,
        //             *ledger_id,
        //             Account {
        //                 owner: from,
        //                 subaccount: None,
        //             },
        //             Account {
        //                 owner: ic_cdk::api::canister_self(),
        //                 subaccount: None,
        //             },
        //         )
        //         .await;

        //         return result;
        //     }
        //     AssetLedgerType::RASSET => {}
        //     AssetLedgerType::ICP => {
        //         let tx_result =
        //             _verify_deposit_in(from, factored_amout, *ledger_id, block_index.unwrap())
        //                 .await;
        //         return tx_result;
        //     }
        // }

        return false;
    }

    pub async fn _send_out(
        &self,
        amount: u128,
        to: Principal,
        token_identifier: Option<String>,
    ) -> bool {
        return true;
        // let Self {
        //     ledger_id,
        //     asset_decimals: decimals,
        //     ledger_type: asset_type,
        // } = self;
        // let factored_amout = apply_precision(amount, 10u128.pow(*decimals));

        // match asset_type {
        //     AssetLedgerType::ICP => {
        //         let tx_result = send_asset_out_icp(
        //             factored_amout,
        //             *ledger_id,
        //             None,
        //             Account {
        //                 owner: to,
        //                 subaccount: None,
        //             },
        //         )
        //         .await;

        //         return tx_result;
        //     }
        //     AssetLedgerType::ICRC => {
        //         let tx_result = send_asset_out_icrc(
        //             factored_amout,
        //             *ledger_id,
        //             None,
        //             Account {
        //                 owner: to,
        //                 subaccount: None,
        //             },
        //         )
        //         .await;
        //         return tx_result;
        //     }
        //     AssetLedgerType::RASSET => {
        //         let tx_result =
        //             _send_out_rassets(to, amount, *ledger_id, token_identifier.unwrap()).await;

        //         return tx_result;
        //     }
        // }
    }
}

async fn _send_out_rassets(
    sender: Principal,
    deposit_amount: u128,
    ledger_id: Principal,
    token_identifier: String,
) -> bool {
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
async fn send_asset_out_icp(
    amount: Amount,
    ledger_id: Principal,
    from_sub: Option<Subaccount>,
    to_account: Account,
) -> bool {
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

/// Verify Deposit In
///
///@dev Because the ICP ledger doesn't have  the transferFrom feature ,we can only verify transaction blocks on the ICP
/// This is a custom implementation that verifies a deposit by using the block index of the transaction of the transaction
/// so after user transfers icp to the canister principal null account (i.e AccountIdentifier with the default subaccount)
/// the transaction returns the block index ,the user then cllas the deposit function  on this canister  with the block index as argument
/// the cansiter verifies that the block has not been used and calls the ledger canister to verify the transaction
async fn _verify_deposit_in(
    sender: Principal,
    deposit_amount: u128,
    ledger_id: Principal,
    block_index: BlockIndex,
) -> bool {
    let args = GetBlocksArgs {
        start: block_index,
        length: 1,
    };
    let block = query_blocks(ledger_id, &args).await;
    match block {
        Ok(response) => {
            let block = if response.blocks.get(0).is_some() {
                response.blocks.get(0).unwrap().clone()
            } else {
                return false;
            };

            let Transaction { operation, .. } = block.transaction;

            let op = if operation.is_some() {
                operation.unwrap()
            } else {
                return false;
            };
            if let Operation::Transfer {
                from, to, amount, ..
            } = op
            {
                let verification = AccountIdentifier::new(&sender, &ICSubaccount([0; 32])) == from
                    && amount == Tokens::from_e8s(deposit_amount as u64)
                    && AccountIdentifier::new(
                        &(ic_cdk::api::canister_self()),
                        &ICSubaccount([0; 32]),
                    ) == to;

                return verification;
            } else {
                return false;
            }
        }
        Err(_) => return false,
    }
}

fn _to_ic_subaccount(sub: Option<Subaccount>) -> ICSubaccount {
    match sub {
        Some(res) => return ICSubaccount(res),
        None => return DEFAULT_SUBACCOUNT,
    }
}

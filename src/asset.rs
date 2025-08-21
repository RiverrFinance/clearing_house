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

use crate::math::math::apply_precision;

type Amount = u128;

#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
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

#[derive(CandidType, Deserialize)]
pub struct GetExchangeRateRequest {
    /// The base asset, i.e., the first asset in a currency pair. For example,
    /// ICP is the base asset in the currency pair ICP/USD.
    pub base_asset: AssetPricingDetails,
    /// The quote asset, i.e., the second asset in a currency pair. For example,
    /// USD is the quote asset in the currency pair ICP/USD.
    pub quote_asset: AssetPricingDetails,
    /// An optional parameter used to find a rate at a specific time.
    pub timestamp: Option<u64>,
}

#[derive(CandidType, Default, Deserialize)]
pub struct ExchangeRateMetadata {
    /// The scaling factor for the exchange rate and the standard deviation.
    pub decimals: u32,
    /// The number of queried exchanges for the base asset.
    pub base_asset_num_queried_sources: usize,
    /// The number of rates successfully received from the queried sources for the base asset.
    pub base_asset_num_received_rates: usize,
    /// The number of queried exchanges for the quote asset.
    pub quote_asset_num_queried_sources: usize,
    /// The number of rates successfully received from the queried sources for the quote asset.
    pub quote_asset_num_received_rates: usize,
    /// The standard deviation of the received rates, scaled by the factor `10^decimals`.
    pub standard_deviation: u64,
    /// The timestamp of the beginning of the day for which the forex rates were retrieved, if any.
    pub forex_timestamp: Option<u64>,
}

#[derive(CandidType, Default, Deserialize)]
pub struct ExchangeRate {
    /// The base asset.
    pub base_asset: AssetPricingDetails,
    /// The quote asset.
    pub quote_asset: AssetPricingDetails,
    /// The timestamp associated with the returned rate.
    pub timestamp: u64,
    /// The median rate from the received rates, scaled by the factor `10^decimals` in the metadata.
    pub rate: u64,
    /// Metadata providing additional information about the exchange rate calculation.
    pub metadata: ExchangeRateMetadata,
}

#[derive(CandidType, Deserialize)]
pub enum ExchangeRateError {
    /// Returned when the canister receives a call from the anonymous principal.
    AnonymousPrincipalNotAllowed,
    /// Returned when the canister is in process of retrieving a rate from an exchange.
    Pending,
    /// Returned when the base asset rates are not found from the exchanges HTTP outcalls.
    CryptoBaseAssetNotFound,
    /// Returned when the quote asset rates are not found from the exchanges HTTP outcalls.
    CryptoQuoteAssetNotFound,
    /// Returned when the stablecoin rates are not found from the exchanges HTTP outcalls needed for computing a crypto/fiat pair.
    StablecoinRateNotFound,
    /// Returned when there are not enough stablecoin rates to determine the forex/USDT rate.
    StablecoinRateTooFewRates,
    /// Returned when the stablecoin rate is zero.
    StablecoinRateZeroRate,
    /// Returned when a rate for the provided forex asset could not be found at the provided timestamp.
    ForexInvalidTimestamp,
    /// Returned when the forex base asset is found.
    ForexBaseAssetNotFound,
    /// Returned when the forex quote asset is found.
    ForexQuoteAssetNotFound,
    /// Returned when neither forex asset is found.
    ForexAssetsNotFound,
    /// Returned when the caller is not the CMC and there are too many active requests.
    RateLimited,
    /// Returned when the caller does not send enough cycles to make a request.
    NotEnoughCycles,
    /// Returned if too many collected rates deviate substantially.
    InconsistentRatesReceived,
    /// Until candid bug is fixed, new errors after launch will be placed here.
    Other(OtherError),
}

#[derive(CandidType, Deserialize)]
pub struct OtherError {
    /// The identifier for the error that occurred.
    pub code: u32,
    /// A description of the error that occurred.
    pub description: String,
}

pub type GetExchangeRateResult = Result<ExchangeRate, ExchangeRateError>;

pub struct XRC {
    pub canister_id: Principal,
}

impl XRC {
    pub fn init(canister_id: Principal) -> Self {
        XRC { canister_id }
    }

    /// tries to fetch the current exchange rate of the pair and returns the result
    pub async fn _get_exchange_rate(
        &self,
        request: GetExchangeRateRequest,
    ) -> GetExchangeRateResult {
        let call = Call::unbounded_wait(self.canister_id, "get_exchange_rate")
            .with_arg(request)
            .with_cycles(1_000_000_000);

        return call.await.unwrap().candid().unwrap();
    }
}

#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
#[derive(Clone, Serialize, Deserialize, Default, CandidType)]
pub struct AssetPricingDetails {
    /// The symbol/code of the asset.
    pub symbol: String,
    /// The asset class.
    pub class: AssetClass,
}

#[derive(Serialize, Copy, Clone, Deserialize, CandidType)]
pub enum AssetLedgerType {
    ICP,
    ICRC,
    RASSET,
}

#[derive(Serialize, Deserialize, CandidType, Clone)]

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
        let Self {
            ledger_id,
            asset_decimals: decimals,
            ledger_type: asset_type,
        } = self;
        let factored_amout = apply_precision(amount, 10u128.pow(*decimals));

        match asset_type {
            AssetLedgerType::ICRC => {
                let result = send_asset_in_asset_icrc(
                    factored_amout,
                    *ledger_id,
                    Account {
                        owner: from,
                        subaccount: None,
                    },
                    Account {
                        owner: ic_cdk::api::canister_self(),
                        subaccount: None,
                    },
                )
                .await;

                return result;
            }
            AssetLedgerType::RASSET => {}
            AssetLedgerType::ICP => {
                let tx_result =
                    _verify_deposit_in(from, factored_amout, *ledger_id, block_index.unwrap())
                        .await;
                return tx_result;
            }
        }

        return false;
    }

    pub async fn _send_out(
        &self,
        amount: u128,
        to: Principal,
        token_identifier: Option<String>,
    ) -> bool {
        let Self {
            ledger_id,
            asset_decimals: decimals,
            ledger_type: asset_type,
        } = self;
        let factored_amout = apply_precision(amount, 10u128.pow(*decimals));

        match asset_type {
            AssetLedgerType::ICP => {
                let tx_result = send_asset_out_icp(
                    factored_amout,
                    *ledger_id,
                    None,
                    Account {
                        owner: to,
                        subaccount: None,
                    },
                )
                .await;

                return tx_result;
            }
            AssetLedgerType::ICRC => {
                let tx_result = send_asset_out_icrc(
                    factored_amout,
                    *ledger_id,
                    None,
                    Account {
                        owner: to,
                        subaccount: None,
                    },
                )
                .await;
                return tx_result;
            }
            AssetLedgerType::RASSET => {
                let tx_result =
                    _send_out_rassets(to, amount, *ledger_id, token_identifier.unwrap()).await;

                return tx_result;
            }
        }
    }
}

async fn _send_out_rassets(
    sender: Principal,
    deposit_amount: u128,
    ledger_id: Principal,
    token_identifier: String,
) -> bool {
    return false;
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

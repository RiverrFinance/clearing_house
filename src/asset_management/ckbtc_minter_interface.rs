use icrc_ledger_types::{
    icrc1::account::Subaccount,
    icrc2::transfer_from::{TransferFromArgs, TransferFromError},
};
// You may want to manually adjust some of the types.

use candid::{self, CandidType, Decode, Deserialize, Encode, Principal};
use ic_cdk::call::{Call, CallResult as Result};

#[derive(CandidType, Deserialize)]
pub enum Mode {
    RestrictedTo(Vec<Principal>),
    DepositsRestrictedTo(Vec<Principal>),
    ReadOnly,
    GeneralAvailability,
}

#[derive(CandidType, Deserialize)]
pub struct UpgradeArgs {
    pub get_utxos_cache_expiration_seconds: Option<u64>,
    pub kyt_principal: Option<Principal>,
    pub mode: Option<Mode>,
    pub retrieve_btc_min_amount: Option<u64>,
    pub max_time_in_queue_nanos: Option<u64>,
    pub check_fee: Option<u64>,
    pub btc_checker_principal: Option<Principal>,
    pub min_confirmations: Option<u32>,
    pub kyt_fee: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub enum BtcNetwork {
    Mainnet,
    Regtest,
    Testnet,
}

#[derive(CandidType, Deserialize)]
pub struct InitArgs {
    pub get_utxos_cache_expiration_seconds: Option<u64>,
    pub kyt_principal: Option<Principal>,
    pub ecdsa_key_name: String,
    pub mode: Mode,
    pub retrieve_btc_min_amount: u64,
    pub ledger_id: Principal,
    pub max_time_in_queue_nanos: u64,
    pub btc_network: BtcNetwork,
    pub check_fee: Option<u64>,
    pub btc_checker_principal: Option<Principal>,
    pub min_confirmations: Option<u32>,
    pub kyt_fee: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub enum MinterArg {
    Upgrade(Option<UpgradeArgs>),
    Init(InitArgs),
}

#[derive(CandidType, Deserialize)]
pub struct EstimateWithdrawalFeeArg {
    pub amount: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub struct EstimateWithdrawalFeeRet {
    pub minter_fee: u64,
    pub bitcoin_fee: u64,
}

#[derive(CandidType, Deserialize)]
pub struct GetBtcAddressArg {
    pub owner: Option<Principal>,
    pub subaccount: Option<Subaccount>,
}

#[derive(CandidType, Deserialize)]
pub enum CanisterStatusType {
    #[serde(rename = "stopped")]
    Stopped,
    #[serde(rename = "stopping")]
    Stopping,
    #[serde(rename = "running")]
    Running,
}

#[derive(CandidType, Deserialize)]
pub enum LogVisibility {
    #[serde(rename = "controllers")]
    Controllers,
    #[serde(rename = "public")]
    Public,
    #[serde(rename = "allowed_viewers")]
    AllowedViewers(Vec<Principal>),
}

#[derive(CandidType, Deserialize)]
pub struct DefiniteCanisterSettings {
    pub freezing_threshold: candid::Nat,
    pub controllers: Vec<Principal>,
    pub reserved_cycles_limit: candid::Nat,
    pub log_visibility: LogVisibility,
    pub wasm_memory_limit: candid::Nat,
    pub memory_allocation: candid::Nat,
    pub compute_allocation: candid::Nat,
}

#[derive(CandidType, Deserialize)]
pub struct QueryStats {
    pub response_payload_bytes_total: candid::Nat,
    pub num_instructions_total: candid::Nat,
    pub num_calls_total: candid::Nat,
    pub request_payload_bytes_total: candid::Nat,
}

#[derive(CandidType, Deserialize)]
pub struct GetEventsArg {
    pub start: u64,
    pub length: u64,
}

#[derive(CandidType, Deserialize)]
pub struct Account {
    pub owner: Principal,
    pub subaccount: Option<Subaccount>,
}

#[derive(CandidType, Deserialize)]
pub struct UtxoOutpoint {
    pub txid: Subaccount,
    pub vout: u32,
}

#[derive(CandidType, Deserialize)]
pub struct Utxo {
    pub height: u32,
    pub value: u64,
    pub outpoint: UtxoOutpoint,
}

#[derive(CandidType, Deserialize)]
pub enum ReimbursementReason {
    CallFailed,
    TaintedDestination {
        kyt_fee: u64,
        kyt_provider: Principal,
    },
}

#[derive(CandidType, Deserialize)]
pub struct EventTypeSentTransactionChangeOutputInner {
    pub value: u64,
    pub vout: u32,
}

#[derive(CandidType, Deserialize)]
pub enum SuspendedReason {
    ValueTooSmall,
    Quarantined,
}

#[derive(CandidType, Deserialize)]
pub struct EventTypeReplacedTransactionChangeOutput {
    pub value: u64,
    pub vout: u32,
}

#[derive(CandidType, Deserialize)]
pub struct GetKnownUtxosArg {
    pub owner: Option<Principal>,
    pub subaccount: Option<Subaccount>,
}

#[derive(CandidType, Deserialize)]
pub struct MinterInfo {
    pub retrieve_btc_min_amount: u64,
    pub min_confirmations: u32,
    pub kyt_fee: u64,
}

#[derive(CandidType, Deserialize)]
pub struct RetrieveBtcArgs {
    pub address: String,
    pub amount: u64,
}

#[derive(CandidType, Deserialize)]
pub struct RetrieveBtcOk {
    pub block_index: u64,
}

#[derive(CandidType, Deserialize)]
pub enum RetrieveBtcError {
    MalformedAddress(String),
    GenericError {
        error_message: String,
        error_code: u64,
    },
    TemporarilyUnavailable(String),
    AlreadyProcessing,
    AmountTooLow(u64),
    InsufficientFunds {
        balance: u64,
    },
}

#[derive(CandidType, Deserialize)]
pub enum RetrieveBtcRet {
    Ok(RetrieveBtcOk),
    Err(RetrieveBtcError),
}

#[derive(CandidType, Deserialize)]
pub struct RetrieveBtcStatusArg {
    pub block_index: u64,
}

#[derive(CandidType, Deserialize)]
pub enum RetrieveBtcStatus {
    Signing,
    Confirmed { txid: Subaccount },
    Sending { txid: Subaccount },
    AmountTooLow,
    Unknown,
    Submitted { txid: Subaccount },
    Pending,
}

#[derive(CandidType, Deserialize)]
pub struct RetrieveBtcStatusV2Arg {
    pub block_index: u64,
}

#[derive(CandidType, Deserialize)]
pub struct ReimbursementRequest {
    pub account: Account,
    pub amount: u64,
    pub reason: ReimbursementReason,
}

#[derive(CandidType, Deserialize)]
pub struct ReimbursedDeposit {
    pub account: Account,
    pub mint_block_index: u64,
    pub amount: u64,
    pub reason: ReimbursementReason,
}

#[derive(CandidType, Deserialize)]
pub enum RetrieveBtcStatusV2 {
    Signing,
    Confirmed { txid: Subaccount },
    Sending { txid: Subaccount },
    AmountTooLow,
    WillReimburse(ReimbursementRequest),
    Unknown,
    Submitted { txid: Subaccount },
    Reimbursed(ReimbursedDeposit),
    Pending,
}

#[derive(CandidType, Deserialize)]
pub struct RetrieveBtcStatusV2ByAccountRetItem {
    pub block_index: u64,
    pub status_v2: Option<RetrieveBtcStatusV2>,
}

#[derive(CandidType, Deserialize)]
pub struct RetrieveBtcWithApprovalArgs {
    pub from_subaccount: Option<Subaccount>,
    pub address: String,
    pub amount: u64,
}

#[derive(CandidType, Deserialize)]
pub enum RetrieveBtcWithApprovalError {
    MalformedAddress(String),
    GenericError {
        error_message: String,
        error_code: u64,
    },
    TemporarilyUnavailable(String),
    InsufficientAllowance {
        allowance: u64,
    },
    AlreadyProcessing,
    AmountTooLow(u64),
    InsufficientFunds {
        balance: u64,
    },
}

#[derive(CandidType, Deserialize)]
pub enum RetrieveBtcWithApprovalRet {
    Ok(RetrieveBtcOk),
    Err(RetrieveBtcWithApprovalError),
}

#[derive(CandidType, Deserialize)]
pub struct UpdateBalanceArg {
    pub owner: Option<Principal>,
    pub subaccount: Option<Subaccount>,
}

#[derive(CandidType, Deserialize)]
pub enum UtxoStatus {
    ValueTooSmall(Utxo),
    Tainted(Utxo),
    Minted {
        minted_amount: u64,
        block_index: u64,
        utxo: Utxo,
    },
    Checked(Utxo),
}

pub type Timestamp = u64;
#[derive(CandidType, Deserialize)]
pub struct SuspendedUtxo {
    pub utxo: Utxo,
    pub earliest_retry: Timestamp,
    pub reason: SuspendedReason,
}

#[derive(CandidType, Deserialize)]
pub struct PendingUtxoOutpoint {
    pub txid: Subaccount,
    pub vout: u32,
}

#[derive(CandidType, Deserialize)]
pub struct PendingUtxo {
    pub confirmations: u32,
    pub value: u64,
    pub outpoint: PendingUtxoOutpoint,
}

#[derive(CandidType, Deserialize)]
pub enum UpdateBalanceError {
    GenericError {
        error_message: String,
        error_code: u64,
    },
    TemporarilyUnavailable(String),
    AlreadyProcessing,
    NoNewUtxos {
        suspended_utxos: Option<Vec<SuspendedUtxo>>,
        required_confirmations: u32,
        pending_utxos: Option<Vec<PendingUtxo>>,
        current_confirmations: Option<u32>,
    },
}

#[derive(CandidType, Deserialize)]
pub enum UpdateBalanceRet {
    Ok(Vec<UtxoStatus>),
    Err(UpdateBalanceError),
}

pub struct Service(pub Principal);
impl Service {
    // pub async fn estimate_withdrawal_fee(
    //     &self,
    //     arg0: EstimateWithdrawalFeeArg,
    // ) -> Result<(EstimateWithdrawalFeeRet,)> {
    //     Call::unbounded_wait(self.0, "estimate_withdrawal_fee")
    //         .with_arg(arg0)
    //         .await
    //         .unwrap()
    //         .candid()
    //         .unwrap()
    // }
    pub async fn get_btc_address(&self, arg0: GetBtcAddressArg) -> String {
        Call::unbounded_wait(self.0, "get_btc_address")
            .with_arg(arg0)
            .await
            .unwrap()
            .candid()
            .unwrap()
    }

    pub async fn retrieve_btc_with_approval(
        &self,
        arg0: RetrieveBtcWithApprovalArgs,
    ) -> RetrieveBtcWithApprovalRet {
        Call::unbounded_wait(self.0, "get_btc_address")
            .with_arg(arg0)
            .await
            .unwrap()
            .candid()
            .unwrap()
    }
}

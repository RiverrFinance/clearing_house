use candid::CandidType;
use ic_ledger_types::BlockIndex;
use serde::Deserialize;

#[derive(CandidType, Deserialize)]
pub struct DepositParams {
    pub amount: u128,
    pub block_index: Option<BlockIndex>,
}

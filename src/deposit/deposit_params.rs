use candid::CandidType;
use ic_ledger_types::BlockIndex;
use serde::Deserialize;

/// Parameters for depositing assets into a user's account.
///
/// This struct contains the necessary information to deposit assets from the house
/// asset ledger into a user's account balance in the clearing house.
#[derive(CandidType, Deserialize)]
pub struct DepositParams {
    /// The quote asset amount to deposit into the user's account.
    /// Uses 20-decimal precision (e.g., 10000000000000000000000 for 1.0 quote unit).
    /// The amount must correspond to a valid transaction in the house asset ledger.
    pub amount: u128,

    /// Optional block index for transaction verification.
    /// This can be used to reference a specific transaction in the ledger for verification purposes.
    /// If provided, the ledger will verify the transaction exists and is valid.
    #[serde(rename = "blockIndex")]
    pub block_index: Option<BlockIndex>,
}

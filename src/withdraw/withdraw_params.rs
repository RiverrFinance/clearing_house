use candid::CandidType;
use serde::Deserialize;

/// Parameters for withdrawing assets from a user's account.
///
/// This struct contains the necessary information to withdraw assets from a user's
/// account balance in the clearing house to the house asset ledger.
#[derive(CandidType, Deserialize)]
pub struct WithdrawParams {
    /// The amount of assets to withdraw from the user's account.
    /// This should be specified with 20 decimal places precision (e.g., 10000000000000000000000 for 1.0 unit).
    /// The user must have sufficient balance to cover this amount.
    /// If the withdrawal fails, this amount will be refunded to the user's balance.
    pub amount: u128,
}

use candid::CandidType;
use serde::Deserialize;

#[derive(CandidType, Deserialize)]
pub enum ClosePositionResult {
    Settled { returns: u128 },
    Waiting,
    Failed,
}

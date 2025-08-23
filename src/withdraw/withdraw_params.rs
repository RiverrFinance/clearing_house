use candid::CandidType;
use serde::Deserialize;

#[derive(CandidType, Deserialize)]
pub struct WithdrawParams {
    pub amount: u128,
}

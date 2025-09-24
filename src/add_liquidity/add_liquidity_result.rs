use candid::CandidType;
use serde::Deserialize;

#[derive(CandidType, Deserialize)]
pub enum AddedLiquidityResult {
    Settled { amount_out: u128 },
    Waiting, // market index ,priority index,operation id
    Failed(String),
}

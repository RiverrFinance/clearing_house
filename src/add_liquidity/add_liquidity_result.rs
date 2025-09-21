use candid::CandidType;
use serde::Deserialize;

#[derive(CandidType, Deserialize)]
pub enum AddedLiquidityResult {
    Settled { amount_out: u128 },
    Waiting { id: Option<(u64, u8, u64)> }, // market index ,priority index,operation id
    Failed(String),
}

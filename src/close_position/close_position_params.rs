use candid::CandidType;
use serde::Deserialize;

#[derive(CandidType, Deserialize, Copy, Clone)]
pub struct ClosePositionParams {
    pub position_id: u64,
    pub acceptable_price_limit: u128,
}

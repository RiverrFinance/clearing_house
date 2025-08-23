use candid::{CandidType, Principal};
use serde::Deserialize;
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
#[derive(CandidType, Clone, Copy, Deserialize)]
pub struct OpenPositionParams {
    pub owner: Principal,
    pub long: bool,
    pub market_index: u64,
    pub collateral: u128,
    pub leverage_factor: u128,
    pub acceptable_price_limit: u128,
    pub reserve_factor: u128,
}

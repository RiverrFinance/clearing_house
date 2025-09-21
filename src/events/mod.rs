use candid::CandidType;
use serde::Deserialize;

#[derive(CandidType, Deserialize)]
pub enum Events {
    OpenPosition,
    ClosePosition,
    AddedLiquidity,
    RemovedLiquidity,
    CreateOrder,
    CancelOrder,
}

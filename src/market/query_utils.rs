use crate::market::market_details::MarketDetails;
use crate::stable_memory::{MARKET_TIMER_MANAGER, MARKETS};
use ic_cdk::query;

pub fn _get_market_timer_details(market_index: u64) -> u64 {
    MARKET_TIMER_MANAGER.with_borrow(|reference| *(reference.get(&market_index).unwrap()))
}

#[query]
pub fn get_market_details(market_index: u64) -> Option<MarketDetails> {
    MARKETS.with_borrow(|reference| reference.get(market_index))
}

#[query]
pub fn get_markets_count() -> u64 {
    MARKETS.with_borrow(|reference| reference.len())
}

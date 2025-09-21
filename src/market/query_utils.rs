use crate::market::market_details::MarketDetails;
use crate::stable_memory::MARKETS_LIST;

#[ic_cdk::query]
pub fn get_market_details(market_index: u64) -> MarketDetails {
    MARKETS_LIST.with_borrow(|reference| {
        let market = reference.get(market_index).expect("Market does not exist");
        market
    })
}

#[ic_cdk::query]
pub fn get_markets_count_plus_1() -> u64 {
    MARKETS_LIST.with_borrow(|reference| reference.len() + 1)
}

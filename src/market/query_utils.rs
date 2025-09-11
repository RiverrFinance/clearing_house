use crate::market::market_details::MarketDetails;
use crate::stable_memory::MARKETS_WITH_LAST_PRICE_UPDATE_TIME;

pub fn get_market_details(market_index: u64) -> MarketDetails {
    MARKETS_WITH_LAST_PRICE_UPDATE_TIME.with_borrow(|reference| {
        let (market, _) = reference.get(market_index).expect("Market does not exist");
        market
    })
}

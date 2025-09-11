use ic_cdk::update;

use crate::admin_roles::admin_guard;
use crate::stable_memory::MARKETS_WITH_LAST_PRICE_UPDATE_TIME;

#[update(name = "settleFundingFees", guard = "admin_guard")]
fn settle_funding_fees(market_index: u64) {
    MARKETS_WITH_LAST_PRICE_UPDATE_TIME.with_borrow_mut(|reference| {
        let (mut market, last_price_update_time) = reference.get(market_index).unwrap();

        market.settle_funding_payment();

        reference.set(market_index, &(market, last_price_update_time));
    });
}

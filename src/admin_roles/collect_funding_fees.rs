use ic_cdk::update;

use crate::admin_roles::admin_guard;
use crate::stable_memory::MARKETS_LIST;

#[update(name = "settleFundingFees", guard = "admin_guard")]
fn settle_funding_fees(market_index: u64) {
    MARKETS_LIST.with_borrow_mut(|reference| {
        let mut market = reference.get(market_index).expect("Market does not exist");

        market.settle_funding_payment();

        reference.set(market_index, &market);
    });
}

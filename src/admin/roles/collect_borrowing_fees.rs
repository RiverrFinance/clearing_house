use candid::CandidType;
use serde::Deserialize;

use crate::{
    constants::COLLECT_BORROW_FEES_PRIORITY_INDEX,
    market::{market_details::MarketDetails, query_utils::_get_market_timer_details},
    pricing_update_management::{
        price_waiting_operation_trait::PriceWaitingOperation,
        price_waiting_operation_utils::{
            is_within_price_update_interval, put_price_waiting_operation,
        },
    },
    stable_memory::MARKETS_WITH_LAST_PRICE_UPDATE_TIME,
    utils,
};

#[derive(CandidType, Deserialize)]
pub struct CollectBorrowFeesParams {
    pub market_index: u64,
}

impl PriceWaitingOperation for CollectBorrowFeesParams {
    fn execute(&self) {
        _collect_borrow_fees(self.market_index);
    }
}

pub fn collect_borrow_fees(market_index: u64) {
    let outcome = _collect_borrow_fees(market_index);

    if outcome == false {
        put_price_waiting_operation(
            market_index,
            COLLECT_BORROW_FEES_PRIORITY_INDEX,
            Box::new(CollectBorrowFeesParams { market_index }),
        );
    }
}

fn _collect_borrow_fees(market_index: u64) -> bool {
    MARKETS_WITH_LAST_PRICE_UPDATE_TIME.with_borrow_mut(|reference| {
        // let Some((mut market, last_price_update_time) = reference.get(market_index).unwrap()
        let (mut market, last_price_update_time) = reference.get(market_index).unwrap();

        if is_within_price_update_interval(last_price_update_time) == false {
            return false;
        }

        let last_time_updated = _get_market_timer_details(market_index);

        let hours_since_last_updated = utils::duration_in_hours(last_time_updated);

        if hours_since_last_updated >= 8 {
            _collect_funding_fees(&mut market)
        };

        market.collect_borrowing_payment();

        reference.set(market_index, &(market, last_price_update_time));

        return true;
    })
}

//

fn _collect_funding_fees(market: &mut MarketDetails) {
    market.settle_funding_payment();
}

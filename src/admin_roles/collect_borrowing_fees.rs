use candid::CandidType;
use ic_cdk::update;
use serde::Deserialize;

use crate::admin_roles::admin_guard;
use crate::{
    constants::COLLECT_BORROW_FEES_PRIORITY_INDEX,
    market::market_details::MarketDetails,
    pricing_update_management::{
        price_waiting_operation_trait::{PriceWaitingOperation, PriceWaitingOperationTrait},
        price_waiting_operation_utils::{
            is_within_price_update_interval, put_price_waiting_operation,
        },
    },
    stable_memory::MARKETS_WITH_LAST_PRICE_UPDATE_TIME,
};

#[derive(CandidType, Deserialize)]
pub struct CollectBorrowFeesParams {
    #[serde(rename = "marketIndex")]
    pub market_index: u64,
}

impl PriceWaitingOperationTrait for CollectBorrowFeesParams {
    fn execute(&self) {
        _collect_borrow_fees(self.market_index);
    }
}

impl From<CollectBorrowFeesParams> for PriceWaitingOperation {
    fn from(params: CollectBorrowFeesParams) -> Self {
        PriceWaitingOperation::CollectBorrowFees(params)
    }
}

#[update(name = "collectBorrowFees", guard = "admin_guard")]
pub fn collect_borrow_fees(market_index: u64) {
    let outcome = _collect_borrow_fees(market_index);

    if outcome == false {
        put_price_waiting_operation(
            market_index,
            COLLECT_BORROW_FEES_PRIORITY_INDEX,
            PriceWaitingOperation::from(CollectBorrowFeesParams { market_index }),
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

        market.collect_borrowing_payment();

        reference.set(market_index, &(market, last_price_update_time));

        return true;
    })
}

//

fn _collect_funding_fees(market: &mut MarketDetails) {
    market.settle_funding_payment();
}

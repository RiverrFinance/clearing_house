use candid::Principal;
use ic_cdk::update;

use crate::market::{market_details::MarketDetails, query_utils::_get_market_timer_details};
use crate::pricing_update_management::price_waiting_operation_arg_variants::PriceWaitingOperation;
use crate::pricing_update_management::price_waiting_operation_utils::put_price_waiting_operation;
use crate::stable_memory::{ADMIN, MARKETS};
use crate::utils;

fn admin_guard() -> Result<(), String> {
    ADMIN.with_borrow(|admin_ref| {
        let admin = admin_ref.get().clone();
        if ic_cdk::api::msg_caller() == admin {
            return Ok(());
        } else {
            return Err("Invalid".to_string());
        };
    })
}

#[update(name = "addMarket", guard = "admin_guard")]
pub fn add_market(details: MarketDetails) {
    MARKETS.with_borrow_mut(|reference| {
        reference.push(&details);
    })
}

#[update(name = "setAdmin", guard = "admin_guard")]
pub fn set_admin(new_admin: Principal) {
    ADMIN.with_borrow_mut(|reference| reference.set(new_admin));
}

fn collect_borrow_fees(market_index: u64) {
    let last_time_updated = _get_market_timer_details(market_index);

    let hours_since_last_updated = utils::duration_in_hours(last_time_updated);

    if hours_since_last_updated >= 8 {
        _collect_funding_fees(market_index)
    };

    let outcome = _collect_borrow_fees(market_index);

    if outcome == false {
        put_price_waiting_operation(
            market_index,
            PriceWaitingOperation::CollectBorrowingFeesOp,
            false,
        );
    }
}

fn _collect_borrow_fees(market_index: u64) -> bool {
    MARKETS.with_borrow(|reference| {
        let mut market = reference.get(market_index).unwrap();

        let outcome = market.collect_borrowing_payment();

        reference.set(market_index, &market);

        return outcome;
    })
}

fn _collect_funding_fees(market_index: u64) {
    MARKETS.with_borrow_mut(|reference| {
        let mut market = reference.get(market_index).unwrap();

        market.settle_funding_payment();

        reference.set(market_index, &market)
    })
}

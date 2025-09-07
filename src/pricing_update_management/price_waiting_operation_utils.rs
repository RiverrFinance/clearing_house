use std::collections::HashMap;
use std::time::Duration;

use crate::constants::MAX_ALLOWED_PRICE_CHANGE_INTERVAL;
use crate::pricing_update_management::price_fetch::update_price;
use crate::pricing_update_management::price_waiting_operation_trait::PriceWaitingOperation;
use crate::stable_memory::MARKET_PRICE_WAITING_OPERATION;

use ic_cdk::api::time;

pub fn is_within_price_update_interval(last_price_update_time: u64) -> bool {
    let current_time = time();

    return current_time - last_price_update_time <= MAX_ALLOWED_PRICE_CHANGE_INTERVAL;
}

pub fn put_price_waiting_operation(
    market_index: u64,
    operation_priority_index: u8,
    operation: Box<dyn PriceWaitingOperation>,
) {
    let new_timer = ic_cdk_timers::set_timer(Duration::from_millis(500), move || {
        ic_cdk::futures::spawn(async move {
            schedule_execution_of_price_waiting_operations(market_index).await;
        });
    });

    MARKET_PRICE_WAITING_OPERATION.with_borrow_mut(|reference| {
        let (_, operations) = reference
            .entry(market_index)
            .and_modify(|value| {
                let (timer_id, _) = value;
                ic_cdk_timers::clear_timer(*timer_id);

                *timer_id = new_timer;
            })
            .or_insert((new_timer, HashMap::new()));

        operations
            .get_mut(&operation_priority_index)
            .unwrap_or(&mut Vec::new())
            .push(operation);
    });
}

pub async fn schedule_execution_of_price_waiting_operations(market_index: u64) {
    update_price(market_index).await;

    let (_, operations) = MARKET_PRICE_WAITING_OPERATION.with_borrow_mut(|reference| {
        reference
            .remove(&market_index)
            .expect("Market was removed before oepration started")
    });

    for operations in operations.values().into_iter() {
        for operation in operations.iter() {
            operation.execute();
        }
    }
}

use candid::Principal;

use crate::{position::position_details::PositionDetails, stable_memory::USERS_POSITIONS};

#[derive(Debug, Clone)]
pub struct QueryGetUserPositionState {
    pub borrowing_fees_owned: u128,
    pub funding_fees_pay: u128,
    pub liquidation_price: u128,
}

pub fn _get_user_position_details(user: Principal, position_id: u64) -> (u64, PositionDetails) {
    USERS_POSITIONS.with_borrow(|reference| reference.get(&(user, position_id)).unwrap())
}

pub fn _put_user_position_detail(
    user: Principal,
    market_index: u64,
    position_id: u64,
    position_details: PositionDetails,
) {
    USERS_POSITIONS.with_borrow_mut(|reference| {
        reference.insert((user, position_id), (market_index, position_details));
    });
}

pub fn remove_user_position_detail(user: Principal, position_id: u64) {
    USERS_POSITIONS.with_borrow_mut(|reference| {
        reference.remove(&(user, position_id));
    });
}

pub fn try_get_user_position_details(
    user: Principal,
    position_id: u64,
) -> Option<(u64, PositionDetails)> {
    USERS_POSITIONS.with_borrow(|reference| reference.get(&(user, position_id)))
}

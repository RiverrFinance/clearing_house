use crate::stable_memory::USERS_POSITIONS;
use candid::Principal;

use crate::position::position_details::PositionDetails;

pub fn get_user_position_details(user: Principal, position_id: u64) -> (u64, PositionDetails) {
    USERS_POSITIONS.with_borrow(|reference| reference.get(&(user, position_id)).unwrap())
}

pub fn try_get_user_position_details(
    user: Principal,
    position_id: u64,
) -> Option<(u64, PositionDetails)> {
    USERS_POSITIONS.with_borrow(|reference| reference.get(&(user, position_id)))
}

pub mod collect_borrowing_fees;
pub mod collect_funding_fees;
pub mod create_market;

use crate::stable_memory::ADMIN;

pub fn admin_guard() -> Result<(), String> {
    let caller = ic_cdk::api::msg_caller();
    let admin = ADMIN.with_borrow(|reference| *reference.get());
    if caller == admin {
        Ok(())
    } else {
        Err("Caller is not admin".to_string())
    }
}

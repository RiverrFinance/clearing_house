use crate::stable_memory::{USER_MARKET_LIQUIDTY_SHARES_BALANCES, USERS_BALANCES};
use candid::Principal;
use ic_cdk::query;

#[query]
pub fn get_user_balance(user: Principal) -> u128 {
    USERS_BALANCES.with_borrow(|reference| reference.get(&user).unwrap_or_default())
}

#[query]
pub fn get_user_market_liquidity_shares(user: Principal, market_index: u64) -> u128 {
    USER_MARKET_LIQUIDTY_SHARES_BALANCES
        .with_borrow(|reference| reference.get(&(user, market_index)).unwrap_or_default())
}

pub fn update_user_market_liquidity_shares(
    user: Principal,
    market_index: u64,
    amount: u128,
    add: bool,
) {
    USER_MARKET_LIQUIDTY_SHARES_BALANCES.with_borrow_mut(|reference| {
        let current_shares = reference.get(&(user, market_index)).unwrap_or_default();
        if add {
            reference.insert((user, market_index), current_shares + amount)
        } else {
            reference.insert((user, market_index), current_shares - amount)
        }
    });
}

pub fn set_user_balance(user: Principal, amount: u128) {
    USERS_BALANCES.with_borrow_mut(|reference| reference.insert(user, amount));
}

pub fn update_user_balance(user: Principal, amount: u128, add: bool) {
    USERS_BALANCES.with_borrow_mut(|reference| {
        let current_balance = reference.get(&user).unwrap_or_default();
        if add {
            reference.insert(user, current_balance + amount)
        } else {
            reference.insert(user, current_balance - amount)
        }
    });
}

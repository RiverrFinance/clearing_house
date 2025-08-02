use core::time;
use std::cell::{RefCell, RefMut};

use candid::{CandidType, Decode, Encode, Principal};
use ic_cdk::api::msg_caller;
use ic_cdk::caller;

use crate::market::{MarketDetails, MarketState};
use crate::math::_percentage;
use crate::types::{Amount, Asset};
use crate::vault::Vault;

use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell, StableVec};

type Memory = VirtualMemory<DefaultMemoryImpl>;

const ONE_HOUR: u64 = 3_600_000_000_000;

const _ADMIN_MEMORY: MemoryId = MemoryId::new(1);

const _MARKETS_ARRAY_MEMORY: MemoryId = MemoryId::new(2);
const _VAULT_MEMORY: MemoryId = MemoryId::new(3);
const _BALANCES_MEMORY: MemoryId = MemoryId::new(4);

thread_local! {
      static MEMORY_MANAGER:RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(MemoryManager::init(DefaultMemoryImpl::default())) ;

      static MARKETS:RefCell<StableVec<MarketDetails,Memory>> = RefCell::new(StableVec::new(MEMORY_MANAGER.with(|s|{
        s.borrow().get(_MARKETS_ARRAY_MEMORY)
      })));

      static USERS_BALANCES:RefCell<StableBTreeMap<Principal,Amount,Memory>> = RefCell::new(StableBTreeMap::init(MEMORY_MANAGER.with_borrow(|tag|{
        tag.get(_BALANCES_MEMORY)
      })));

      static VAULT:RefCell<StableCell<Vault,Memory>> = RefCell::new(StableCell::init(MEMORY_MANAGER.with_borrow(|tag|{
        tag.get(_VAULT_MEMORY)
      }), Vault::default()))
}

fn add_market(market: MarketDetails) {
    MARKETS.with_borrow_mut(|tag| {
        tag.push(&market);
    })
}

fn open_position(
    market_index: u64,
    margin: u128,
    leverage_x10: u8,
    entry_price: u64,
    slippage: u64,
    max_pnl: u64,
    long: bool,
) {
    MARKETS.with_borrow(|tag| {
        let MarketDetails { price, state, .. } = tag.get(market_index).unwrap();

        let MarketState {
            min_collateral,
            max_leverage_x10,
            max_pnl,
            execution_fee,
            ..
        } = state;

        assert!(margin >= min_collateral, "I");
        assert!(leverage_x10 < max_leverage_x10, "II");
        assert!(max_pnl >= state.max_pnl, "III");

        let user_balance = get_user_balance(msg_caller());

        assert!(user_balance >= margin + execution_fee, "");

        let debt = (u128::from(leverage_x10 - 10) * margin) / 10;

        let max_profit = _percentage(max_pnl, debt + margin);

        if ic_cdk::api::time() - price.last_fetched <= ONE_HOUR {
        } else {
        }

        // let max_units = (max_profit * price.decimals as u128) / (price.price as u128);
    })
}

fn get_user_balance(user: Principal) -> Amount {
    USERS_BALANCES.with_borrow(|tag| tag.get(&user).unwrap_or_default())
}

pub mod bias;
pub mod floatmath;
pub mod funding;
pub mod market;
pub mod math;
pub mod pricing;
pub mod types;
pub mod vault;

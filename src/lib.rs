use std::cell::RefCell;

use candid::{CandidType, Decode, Encode, Principal};
use ic_cdk::call::Call;
use ic_cdk::{export_candid, storage};

use crate::types::{Asset, BiasTracker};

use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell, StableVec};
use ic_stable_structures::{Storable, storable::Bound};

type Memory = VirtualMemory<DefaultMemoryImpl>;

const _ADMIN_MEMORY: MemoryId = MemoryId::new(1);

const _MARKETS_ARRAY_MEMORY: MemoryId = MemoryId::new(2);

thread_local! {
      static MEMORY_MANAGER:RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(MemoryManager::init(DefaultMemoryImpl::default())) ;

      static MARKETS:RefCell<StableVec<u8,Memory>> = RefCell::new(StableVec::new(MEMORY_MANAGER.with(|s|{
        s.borrow().get(_MARKETS_ARRAY_MEMORY)
      })))
}

pub struct MarketDetails {
    base_asset: Asset,
    positions_tracker: BiasTracker,
    price: Price,
    state: State,
}
pub struct State {
    pub max_leverage_x10: u8,
    pub max_price_delta_x10: u8,
    pub min_collateral: u128,
    pub execution_fee: u128,
}

pub struct UserPosition {
    collateral_value_share: u128,
    debt: u128,
    entry_price: u64,
    position_size: u128,
    position_value: u128,
    direction: bool,
    max_gain: u128,
    timestamp: u64,
}

pub enum PositionState {
    PENDING { entry_price: u64 },
    RESLOVED,
}
type Time = u64;

pub struct Price {
    price: u64,
    decimals: u32,
    last_fetched: Time,
}
///
/// put expected price ,target pnl, margin
///
///

#[ic_cdk::update]
async fn open_position(margin: u128, leverage: u128, tp: u64, slippage: u32) {
    let mut name = Vec::new();
    name.push("string".to_string())
}

async fn execute_liquidate(user: Principal) {}
async fn try_liquidate(user: Principal) {}

pub mod math;
pub mod types;

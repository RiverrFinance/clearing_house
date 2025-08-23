use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};

use candid::Principal;
use ic_cdk_timers::TimerId;

use crate::constants::{
    _ADMIN_MEMORY, _BALANCES_MEMORY, _HOUSE_DETAILS_MEMORY, _MARKET_LIQUIDTY_SHARES_MEMORY,
    _MARKETS_MEMORY, _XRC_MEMORY,
};

use crate::house_settings::HouseDetails;
use crate::market::market_details::MarketDetails;
use crate::position::position_details::PositionDetails;
use crate::pricing_update_management::price_waiting_operation_arg_variants::PriceWaitingOperation;

use ic_stable_structures::memory_manager::{MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell, StableVec};

type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
      static MEMORY_MANAGER:RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(MemoryManager::init(DefaultMemoryImpl::default())) ;

    pub  static ADMIN:RefCell<StableCell<Principal,Memory>> = RefCell::new(StableCell::init(MEMORY_MANAGER.with_borrow(|tag|{
        tag.get(_ADMIN_MEMORY)
      }),Principal::anonymous()));

    pub static HOUSE_SETTINGS:RefCell<StableCell<HouseDetails,Memory>> = RefCell::new(StableCell::init(MEMORY_MANAGER.with_borrow(|tag|{
        tag.get(_HOUSE_DETAILS_MEMORY)
      }), HouseDetails::default()));


    pub static XRC:RefCell<StableCell<Principal,Memory>> = RefCell::new(StableCell::init(MEMORY_MANAGER.with_borrow(|tag|{
        tag.get(_XRC_MEMORY)
      }), Principal::anonymous()));

    pub static MARKETS:RefCell<StableVec<MarketDetails,Memory>> = RefCell::new(StableVec::new(MEMORY_MANAGER.with(|s|{
        s.borrow().get(_MARKETS_MEMORY)
      })));

     pub static USERS_BALANCES:RefCell<StableBTreeMap<Principal,u128,Memory>> = RefCell::new(StableBTreeMap::init(MEMORY_MANAGER.with_borrow(|tag|{
        tag.get(_BALANCES_MEMORY)
      })));

      pub static  USER_MARKET_LIQUIDTY_SHARES_BALANCES:RefCell<StableBTreeMap<(Principal,u64),u128,Memory>> = RefCell::new(StableBTreeMap::init(MEMORY_MANAGER.with_borrow(|tag|{
        tag.get(_MARKET_LIQUIDTY_SHARES_MEMORY)
      })));


    /// User amd TimeStamp

    pub static USERS_POSITIONS:RefCell<StableBTreeMap<(Principal,u64),(u64,PositionDetails),Memory>> = RefCell::new(StableBTreeMap::new(MEMORY_MANAGER.with(|s|{
        s.borrow().get(_MARKETS_MEMORY)
      })));

    pub  static MARKET_PRICE_WAITING_OPERATION:RefCell<HashMap<u64,(TimerId,VecDeque<PriceWaitingOperation>)>> = RefCell::new(HashMap::new());


    pub static MARKET_SHARE_USER_BALANCES:RefCell<HashMap<(Principal,u64),u128>> = RefCell::new(HashMap::new());


    pub static MARKET_TIMER_MANAGER:RefCell<HashMap<u64,u64>> = RefCell::new(HashMap::new());


}

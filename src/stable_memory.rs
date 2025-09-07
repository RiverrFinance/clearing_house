use std::cell::RefCell;
use std::collections::HashMap;

use candid::Principal;
use ic_cdk_timers::TimerId;

use crate::constants::{
    _ADMIN_MEMORY_ID, _BALANCES_MEMORY_ID, _HOUSE_DETAILS_MEMORY_ID,
    _MARKET_LIQUIDTY_SHARES_MEMORY_ID, _MARKETS_MEMORY_ID,
};

use crate::house_settings::HouseDetails;
use crate::market::market_details::MarketDetails;
use crate::position::position_details::PositionDetails;
use crate::pricing_update_management::price_waiting_operation_trait::PriceWaitingOperation;

use ic_stable_structures::memory_manager::{MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell, StableVec};

type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
      static MEMORY_MANAGER:RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(MemoryManager::init(DefaultMemoryImpl::default())) ;

    pub  static ADMIN:RefCell<StableCell<Principal,Memory>> = RefCell::new(StableCell::init(MEMORY_MANAGER.with_borrow(|tag|{
        tag.get(_ADMIN_MEMORY_ID)
      }),Principal::anonymous()));

    pub static HOUSE_SETTINGS:RefCell<StableCell<HouseDetails,Memory>> = RefCell::new(StableCell::init(MEMORY_MANAGER.with_borrow(|tag|{
        tag.get(_HOUSE_DETAILS_MEMORY_ID)
      }), HouseDetails::default()));

    pub static MARKETS_WITH_LAST_PRICE_UPDATE_TIME:RefCell<StableVec<(MarketDetails,u64),Memory>> = RefCell::new(StableVec::new(MEMORY_MANAGER.with(|s|{
        s.borrow().get(_MARKETS_MEMORY_ID)
      })));

     pub static USERS_BALANCES:RefCell<StableBTreeMap<Principal,u128,Memory>> = RefCell::new(StableBTreeMap::init(MEMORY_MANAGER.with_borrow(|tag|{
        tag.get(_BALANCES_MEMORY_ID)
      })));

      pub static  USER_MARKET_LIQUIDTY_SHARES_BALANCES:RefCell<StableBTreeMap<(Principal,u64),u128,Memory>> = RefCell::new(StableBTreeMap::init(MEMORY_MANAGER.with_borrow(|tag|{
        tag.get(_MARKET_LIQUIDTY_SHARES_MEMORY_ID)
      })));


    /// User amd TimeStamp

    pub static USERS_POSITIONS:RefCell<StableBTreeMap<(Principal,u64),(u64,PositionDetails),Memory>> = RefCell::new(StableBTreeMap::new(MEMORY_MANAGER.with(|s|{
        s.borrow().get(_MARKETS_MEMORY_ID)
      })));


    pub  static MARKET_PRICE_WAITING_OPERATION:RefCell<HashMap<u64,(TimerId,HashMap<u8,Vec<Box<dyn PriceWaitingOperation> >>)>> = RefCell::new(HashMap::new());
    pub static MARKET_SHARE_USER_BALANCES:RefCell<HashMap<(Principal,u64),u128>> = RefCell::new(HashMap::new());


    pub static MARKET_TIMER_MANAGER:RefCell<HashMap<u64,u64>> = RefCell::new(HashMap::new());


}

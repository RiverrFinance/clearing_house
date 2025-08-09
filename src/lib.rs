use core::time;
use std::borrow::Cow;
use std::cell::{RefCell, RefMut};
use std::collections::{HashMap, VecDeque};
use std::time::Duration;

use candid::{CandidType, Principal};
use ic_cdk::api::msg_caller;
use ic_cdk::call::Call;

use ic_stable_structures::storable::Bound;
use serde::{Deserialize, Serialize};

use crate::market::{MarketDetails, MarketState, OpenPositionResult, Position};
use crate::math::_percentage;
use crate::types::{Amount, Asset, GetExchangeRateRequest, GetExchangeRateResult, XRC};
use crate::vault::Vault;

use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell, StableVec, Storable};

type Memory = VirtualMemory<DefaultMemoryImpl>;

const _ONE_SECOND: u64 = 1_000_000_000;

const ONE_HOUR: u64 = 3_600_000_000_000;

const _ADMIN_MEMORY: MemoryId = MemoryId::new(1);

const _MARKETS_ARRAY_MEMORY: MemoryId = MemoryId::new(2);
const _VAULT_MEMORY: MemoryId = MemoryId::new(3);
const _BALANCES_MEMORY: MemoryId = MemoryId::new(4);
const _POSITIONS_MEMORY: MemoryId = MemoryId::new(5);

#[derive(Serialize, Deserialize, Clone)]
pub struct Token {
    pub asset_details: Asset,
    pub canister_id: Principal,
    pub decimals: u32,
}

impl Default for Token {
    fn default() -> Self {
        Self {
            asset_details: Asset::default(),
            canister_id: Principal::anonymous(),
            decimals: 0,
        }
    }
}

impl Storable for Token {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let serialized = bincode::serialize(self).expect("failed to serialize");
        Cow::Owned(serialized)
    }

    /// Converts the element into an owned byte vector.
    ///
    /// This method consumes `self` and avoids cloning when possible.
    fn into_bytes(self) -> Vec<u8> {
        bincode::serialize(&self).expect("failed to serialize")
    }

    /// Converts bytes into an element.
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        bincode::deserialize(bytes.as_ref()).expect("failed to desearalize")
    }

    /// The size bounds of the type.
    const BOUND: Bound = Bound::Unbounded;
}

thread_local! {
      static MEMORY_MANAGER:RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(MemoryManager::init(DefaultMemoryImpl::default())) ;

      static MARKETS:RefCell<StableVec<MarketDetails,Memory>> = RefCell::new(StableVec::new(MEMORY_MANAGER.with(|s|{
        s.borrow().get(_MARKETS_ARRAY_MEMORY)
      })));

      static USERS_BALANCES:RefCell<StableBTreeMap<Principal,Amount,Memory>> = RefCell::new(StableBTreeMap::init(MEMORY_MANAGER.with_borrow(|tag|{
        tag.get(_BALANCES_MEMORY)
      })));


      static XRC:RefCell<StableCell<Principal,Memory>> = RefCell::new(StableCell::init(MEMORY_MANAGER.with_borrow(|tag|{
        tag.get(_VAULT_MEMORY)
      }), Principal::anonymous()));

      static HOUSE_ASSET:RefCell<StableCell<Token,Memory>> = RefCell::new(StableCell::init(MEMORY_MANAGER.with_borrow(|tag|{
        tag.get(_VAULT_MEMORY)
      }), Token::default()));

    static POSITIONS:RefCell<HashMap<Principal,Vec<Position>>> = RefCell::new(HashMap::new());

    static WAITING_POSITIONS:RefCell<HashMap<u64,VecDeque<(u128,Position)>>> = RefCell::new(HashMap::new());

    static PRICING_TIMER:RefCell<u64> = RefCell::new(0);
}

pub fn _open_position(
    owner: Principal,
    market_index: u64,
    collateral: u128,
    leverage_x10: u8,
    acceptable_price: u128,
    max_pnl: u128,
    long: bool,
) -> OpenPositionResult {
    MARKETS.with_borrow(|tag| {
        let mut market = tag.get(market_index).unwrap();

        let MarketState {
            min_collateral,
            max_leverage_x10,
            max_pnl_percentage,
            execution_fee,
            ..
        } = market.state;

        let user_balance = get_user_balance(msg_caller());

        if collateral >= min_collateral
            || leverage_x10 < max_leverage_x10
            || user_balance >= collateral + execution_fee
        {
            return OpenPositionResult::Failed;
        }

        // assert!(collateral >= min_collateral, "I");
        // assert!(leverage_x10 < max_leverage_x10, "II");
        // // assert!(market_max_pnl >= max_pnl, "III");

        // let user_balance = get_user_balance(msg_caller());

        // assert!(user_balance >= collateral + execution_fee, "");

        let debt = (u128::from(leverage_x10 - 10) * collateral) / 10;

        return market.open_position(owner, collateral, debt, long, max_pnl, acceptable_price);
    })
}

async fn schdule_execution_of_wait_orders(market_index: u64) {
    _update_price(market_index).await;

    let positions =
        WAITING_POSITIONS.with_borrow_mut(|reference| reference.remove(&market_index).unwrap());

    let mut index = 0;

    while index < positions.len() {
        let (acceptable_price, initial_position) = positions.get(index).unwrap();
        let Position {
            owner,
            collateral,
            debt,
            long,
            max_reserve,
            ..
        } = *initial_position;

        let result = _open_position(
            owner,
            market_index,
            collateral,
            (((debt + collateral) * 10) / collateral) as u8,
            *acceptable_price,
            max_reserve,
            long,
        );
        if let OpenPositionResult::Settled { position } = result {
            _put_position(owner, position);
        }
        index += 1;
    }
}

async fn _update_price(market_index: u64) {
    let mut market = MARKETS.with_borrow(|reference| reference.get(market_index).unwrap());
    let quote_asset = _get_house_asset_details().asset_details;
    let base_asset = market.base_asset();
    let xrc_canister = _get_xrc_id();
    let request = GetExchangeRateRequest {
        base_asset,
        quote_asset,
        timestamp: None,
    };

    let call = Call::unbounded_wait(xrc_canister, "get_exchange_rate")
        .with_arg(request)
        .with_cycles(1_000_000_000);

    let result: GetExchangeRateResult = call.await.unwrap().candid().unwrap();
    if let Ok(response) = result {
        market._update_price(response.rate, response.metadata.decimals);
    }
}

fn _put_position(owner: Principal, position: Position) {
    POSITIONS.with_borrow_mut(|reference| {
        let mut positions = reference.remove(&owner).unwrap_or_default();
        positions.push(position);
        reference.insert(owner, positions);
    })
}

fn _put_waiting_position(market_index: u64, acceptable_price_cap: u128, position: Position) {
    WAITING_POSITIONS.with_borrow_mut(|reference| {
        let mut waiting_positions = reference.remove(&market_index).unwrap_or_default();
        waiting_positions.push_back((acceptable_price_cap, position));

        reference.insert(market_index, waiting_positions);
    })
}

fn _get_xrc_id() -> Principal {
    XRC.with_borrow(|reference| reference.get().clone())
}
fn _get_house_asset_details() -> Token {
    HOUSE_ASSET.with_borrow(|reference| return reference.get().clone())
}

fn _get_pricing_timer() -> u64 {
    PRICING_TIMER.with_borrow(|x| *x)
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

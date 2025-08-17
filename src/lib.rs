use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::time::Duration;

use candid::Principal;
use ic_cdk::api::{msg_caller, time};
use ic_cdk::call::Call;
use ic_cdk::{export_candid, update};
use ic_cdk_timers::TimerId;
use ic_ledger_types::BlockIndex;

use crate::asset::{AssetLedger, AssetPricingDetails};
use crate::constants::ONE_HOUR_NANOSECONDS;
use crate::market::{
    ClosePositionResult, LiquidityOperationResult, MarketDetails, OpenPositionResult,
};
use crate::math::math::{apply_precision, to_precision};
use crate::position::Position;
use crate::types::{Amount, GetExchangeRateRequest, GetExchangeRateResult, HouseDetails};

use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell, StableVec};

type Memory = VirtualMemory<DefaultMemoryImpl>;

const _ONE_SECOND: u64 = 1_000_000_000;

const _ADMIN_MEMORY: MemoryId = MemoryId::new(1);

const _MARKETS_ARRAY_MEMORY: MemoryId = MemoryId::new(2);
const _VAULT_MEMORY: MemoryId = MemoryId::new(3);
const _BALANCES_MEMORY: MemoryId = MemoryId::new(4);
const _POSITIONS_MEMORY: MemoryId = MemoryId::new(5);

thread_local! {
      static MEMORY_MANAGER:RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(MemoryManager::init(DefaultMemoryImpl::default())) ;

     static ADMIN:RefCell<StableCell<Principal,Memory>> = RefCell::new(StableCell::init(MEMORY_MANAGER.with_borrow(|tag|{
        tag.get(_VAULT_MEMORY)
      }),Principal::anonymous()));

      static MARKETS:RefCell<StableVec<MarketDetails,Memory>> = RefCell::new(StableVec::new(MEMORY_MANAGER.with(|s|{
        s.borrow().get(_MARKETS_ARRAY_MEMORY)
      })));

      static USERS_BALANCES:RefCell<StableBTreeMap<Principal,Amount,Memory>> = RefCell::new(StableBTreeMap::init(MEMORY_MANAGER.with_borrow(|tag|{
        tag.get(_BALANCES_MEMORY)
      })));


      static XRC:RefCell<StableCell<Principal,Memory>> = RefCell::new(StableCell::init(MEMORY_MANAGER.with_borrow(|tag|{
        tag.get(_VAULT_MEMORY)
      }), Principal::anonymous()));

      static HOUSE_DETAILS:RefCell<StableCell<HouseDetails,Memory>> = RefCell::new(StableCell::init(MEMORY_MANAGER.with_borrow(|tag|{
        tag.get(_VAULT_MEMORY)
      }), HouseDetails::default()));

    /// User amd TimeStamp

    static USERS_POSITIONS:RefCell<StableBTreeMap<(Principal,u64),(u64,Position),Memory>> = RefCell::new(StableBTreeMap::new(MEMORY_MANAGER.with(|s|{
        s.borrow().get(_MARKETS_ARRAY_MEMORY)
      })));

    static MARKET_PRICE_WAITING_OPERATION:RefCell<HashMap<u64,(TimerId,VecDeque<PriceWaitingOperation>)>> = RefCell::new(HashMap::new());


    static MARKET_SHARE_USER_BALANCES:RefCell<HashMap<(Principal,u64),u128>> = RefCell::new(HashMap::new());


    static MARKET_TIMER_MANAGER:RefCell<HashMap<u64,u64>> = RefCell::new(HashMap::new());


}

/// Deposit function
///
/// Paramters
/// Amount:The amount of house asset to deposit with PRECISION ( see math/math.rs)
/// Block Index :Optional parameter for block index ,utilized for  deposit of ICP token after sending to canister is
/// (@dev see _verify_deposit_in function in asset.rs)
#[ic_cdk::update(name = "depositAsset")]
pub async fn deposit_asset(amount: u128, block_index: Option<BlockIndex>) {
    let user = msg_caller();

    let HouseDetails {
        house_asset_ledger, ..
    } = _get_house_details();

    let tx_result = house_asset_ledger
        ._send_in(amount, user, block_index, None)
        .await;

    if tx_result {
        let user_balance = get_user_balance(user);

        update_user_balance(user, user_balance + amount);
    }
}

#[ic_cdk::update(name = "withdrawAsset")]
pub async fn withdraw_asset(amount: u128) {
    let user = msg_caller();

    let HouseDetails {
        house_asset_ledger, ..
    } = _get_house_details();

    let user_balance = get_user_balance(user);

    if user_balance > amount {
        update_user_balance(user, user_balance - amount);
        let tx_result = house_asset_ledger._send_out(amount, user, None).await;
        if tx_result == false {
            // refund amount back
            update_user_balance(user, user_balance + amount);
        }
    }
}

#[update(name = "depositLiquidity")]
async fn deposit_liquidity(market_index: u64, amount: u128, min_amount_out: u128) {
    let depositor = msg_caller();

    let user_balance = get_user_balance(depositor);

    let execution_fee = _get_execution_fee();

    if amount + execution_fee > user_balance {
        return;
    }

    update_user_balance(depositor, user_balance - execution_fee);
    let result = _deposit_liquidity(market_index, depositor, amount, min_amount_out).await;

    match result {
        LiquidityOperationResult::Settled { amount_out: _ } => {}
        LiquidityOperationResult::Waiting => {
            let new_timer_id = _initiate_scheduling_for_price_wait_operation(market_index);
            _put_waiting_position(
                market_index,
                new_timer_id,
                PriceWaitingOperation::MarketLiquidityOp {
                    depositor,
                    deposit: amount,
                    min_amount_out: min_amount_out,
                },
                false,
            );
        }
        LiquidityOperationResult::Failed => {
            update_user_balance(depositor, user_balance + execution_fee);
        }
    }

    // let tx_result =
}

/// Open Position function
///
/// Parameters
///
/// LONG:true for long and false for a short
/// MARKET_INDEX: The market index of the respective market
/// COLLLATERAL :The amount set as collateral for opening position
/// LEVERAGE_X10 :The leverage to take multiplied by 10
/// ACCEPTABLE_PRICE_LIMIT - The limit price allowed for closing position also correpsonds to maximum slippage price
/// MAX_PNL - This serves as the max reserve for the position
///
/// @dev MAX_PNL serves as a virtual take profit point see README.md file for proper documentation
///
#[update(name = "openPosition")]
pub fn open_position(
    long: bool,
    market_index: u64,
    collateral: u128,
    leverage: u128,
    acceptable_price_limit: u128,
    max_pnl: u128,
) {
    let user = msg_caller();

    let user_balance = get_user_balance(user);

    let execution_fee = _get_execution_fee();

    assert!(user_balance > execution_fee + collateral);

    update_user_balance(user, user_balance - execution_fee);

    let result = _open_position(
        user,
        market_index,
        collateral,
        leverage,
        acceptable_price_limit,
        max_pnl,
        long,
    );

    match result {
        OpenPositionResult::Settled { position } => {
            put_user_position(user, market_index, position);
        }
        OpenPositionResult::Waiting { position } => {
            let new_timer_id = _initiate_scheduling_for_price_wait_operation(market_index);

            _put_waiting_position(
                market_index,
                new_timer_id,
                PriceWaitingOperation::OpenPositionOp(acceptable_price_limit, position),
                true,
            );

            return;
        }
        OpenPositionResult::Failed => {
            return;
        }
    }

    update_user_balance(user, user_balance - collateral);
}

///
///Close Position Function
///
/// Params
/// ID - The ID correponding to the user's position
/// ACCEPTABLE_PRICE_LIMIT - The limit price allowed for closing position also correpsonds to maximum slippage price
#[update(name = "closePosition")]
pub fn close_position(position_id: u64, acceptable_price_limit: u128) {
    let user = msg_caller();

    let (market_index, position) = _get_user_position_details(user, position_id);

    let user_balance = get_user_balance(user);

    match _close_position(market_index, position, acceptable_price_limit) {
        ClosePositionResult::Settled { returns } => {
            update_user_balance(user, user_balance + returns);
            _remove_user_position_details(user, position_id);
        }
        ClosePositionResult::Waiting { position } => {
            let new_timer_id = _initiate_scheduling_for_price_wait_operation(market_index);

            _put_waiting_position(
                market_index,
                new_timer_id,
                PriceWaitingOperation::ClosePositionOp(acceptable_price_limit, position),
                false,
            );
        }
        ClosePositionResult::Failed => {
            return;
        }
    }
}

pub fn _close_position(
    market_index: u64,
    position: Position,
    acceptable_price_limit: u128,
) -> ClosePositionResult {
    MARKETS.with_borrow_mut(|reference| {
        let mut market = reference.get(market_index).expect("Market does not exist");

        let result = market.close_position(position, acceptable_price_limit);

        reference.set(market_index, &market);
        return result;
    })
}

pub fn _open_position(
    owner: Principal,
    market_index: u64,
    collateral: u128,
    leverage: u128,
    acceptable_price_limit: u128,
    max_pnl: u128,
    long: bool,
) -> OpenPositionResult {
    MARKETS.with_borrow_mut(|reference| {
        let mut market = reference.get(market_index).unwrap();

        let debt = apply_precision(leverage, collateral) - collateral;

        let user_balance = get_user_balance(msg_caller());

        if user_balance < collateral {
            return OpenPositionResult::Failed;
        }

        let result = market.open_position(
            owner,
            collateral,
            debt,
            long,
            max_pnl,
            acceptable_price_limit,
        );

        reference.set(market_index, &market);

        return result;
    })
}

async fn _deposit_liquidity(
    market_index: u64,
    depositor: Principal,
    amount: u128,
    min_amount_out: u128,
) -> LiquidityOperationResult {
    let user_balance = get_user_balance(depositor);

    if user_balance < amount {
        return LiquidityOperationResult::Failed;
    }
    let (result, market) = MARKETS.with_borrow_mut(|reference| {
        let mut market = reference.get(market_index).unwrap();

        let result = market.deposit_liquidity(amount, min_amount_out);

        (result, market)
    });

    if let LiquidityOperationResult::Settled { amount_out } = result {
        // reduce balance
        update_user_balance(depositor, user_balance - amount);
        let HouseDetails {
            markets_tokens_ledger,
            ..
        } = _get_house_details();

        let tx_result = markets_tokens_ledger
            ._send_out(amount_out, depositor, Some(market.token_identifier.clone()))
            .await;

        if tx_result == false {
            // refund user
            update_user_balance(depositor, user_balance + amount);
            return LiquidityOperationResult::Failed;
        }
        MARKETS.with_borrow_mut(|reference| {
            // update market
            reference.set(market_index, &market);
        })
    };

    return result;
}

fn _collect_funding_fees(market_index: u64) {
    MARKETS.with_borrow_mut(|reference| {
        let mut market = reference.get(market_index).unwrap();

        market.settle_funding_payment();

        reference.set(market_index, &market)
    })
}

fn _collect_borrow_fees(market_index: u64) -> bool {
    MARKETS.with_borrow(|reference| {
        let mut market = reference.get(market_index).unwrap();

        let outcome = market.collect_borrowing_payment();

        reference.set(market_index, &market);

        return outcome;
    })
}

fn _deposit_liquidity_to_market(
    market_index: u64,
    deposit_amount: u128,
    min_out: u128,
) -> LiquidityOperationResult {
    MARKETS.with_borrow_mut(|reference| {
        let mut market = reference.get(market_index).unwrap();

        let outcome = market.deposit_liquidity(deposit_amount, min_out);

        reference.set(market_index, &market);

        return outcome;
    })
}

async fn schedule_execution_of_wait_operations(market_index: u64) {
    _update_price(market_index).await;

    let (_, operations) = MARKET_PRICE_WAITING_OPERATION
        .with_borrow_mut(|reference| reference.remove(&market_index).unwrap());

    let mut index = 0;

    while index < operations.len() {
        let op = operations.get(index).unwrap();

        match op {
            PriceWaitingOperation::ClosePositionOp(acceptable_price_limit, position) => {
                _close_position(market_index, *position, *acceptable_price_limit);
            }
            PriceWaitingOperation::OpenPositionOp(acceptable_price_limit, position) => {
                let Position {
                    owner,
                    collateral,
                    debt,
                    long,
                    max_reserve,
                    ..
                } = *position;

                let leverage = to_precision(debt + collateral, collateral);

                let result = _open_position(
                    owner,
                    market_index,
                    collateral,
                    leverage,
                    *acceptable_price_limit,
                    max_reserve,
                    long,
                );
                if let OpenPositionResult::Settled { position } = result {
                    put_user_position(owner, market_index, position);
                }
            }
            PriceWaitingOperation::CollectBorrowingFeesOp => {
                _collect_borrow_fees(market_index);
            }
            PriceWaitingOperation::MarketLiquidityOp {
                deposit,
                min_amount_out: min_received,
                depositor,
            } => {
                let _ = _deposit_liquidity(market_index, *depositor, *deposit, *min_received).await;
            }
        }

        index += 1;
    }
}

fn put_user_position(owner: Principal, market_index: u64, position: Position) {
    USERS_POSITIONS.with_borrow_mut(|reference| {
        let id = time();
        reference.insert((owner, id), (market_index, position));
    })
}

fn _get_user_position_details(owner: Principal, id: u64) -> (u64, Position) {
    USERS_POSITIONS.with_borrow(|reference| reference.get(&(owner, id)).unwrap())
}

fn _get_market_timer_details(market_index: u64) -> u64 {
    MARKET_TIMER_MANAGER.with_borrow(|reference| *(reference.get(&market_index).unwrap()))
}

fn _remove_user_position_details(owner: Principal, id: u64) {
    USERS_POSITIONS.with_borrow_mut(|reference| reference.remove(&(owner, id)));
}

fn _put_waiting_position(
    market_index: u64,
    timer_id: TimerId,
    op: PriceWaitingOperation,
    back: bool,
) {
    MARKET_PRICE_WAITING_OPERATION.with_borrow_mut(|reference| {
        let (current_timer_id, operations) = reference.get_mut(&market_index).unwrap();
        *current_timer_id = timer_id;
        if back {
            operations.push_back(op);
        } else {
            operations.push_front(op);
        }
    })
}

fn _get_market_timer_detail(market_index: u64) -> TimerId {
    MARKET_PRICE_WAITING_OPERATION.with_borrow(|reference| {
        let (current_timer_id, _) = reference.get(&market_index).unwrap();
        *current_timer_id
    })
}

fn _set_market_timer_detail(market_index: u64) {
    MARKET_TIMER_MANAGER.with_borrow_mut(|reference| {
        let time = time();
        reference.insert(market_index, time);
    })
}

fn _get_execution_fee() -> u128 {
    HOUSE_DETAILS.with_borrow(|reference| reference.get().execution_fee)
}

fn _get_house_asset_pricing_details() -> AssetPricingDetails {
    HOUSE_DETAILS.with_borrow(|reference| reference.get().house_asset_pricing_details.clone())
}

fn _get_house_details() -> HouseDetails {
    HOUSE_DETAILS.with_borrow(|reference| reference.get().clone())
}

fn _get_markets_tokens_ledger_id() -> AssetLedger {
    HOUSE_DETAILS.with_borrow(|reference| reference.get().markets_tokens_ledger.clone())
}

fn _get_house_asset_decimals() -> u32 {
    HOUSE_DETAILS.with_borrow(|reference| reference.get().house_asset_ledger.asset_decimals)
}

fn _get_admin() -> Principal {
    ADMIN.with_borrow(|reference| *(reference.get()))
}

fn _get_xrc_id() -> Principal {
    XRC.with_borrow(|reference| reference.get().clone())
}

fn get_user_balance(user: Principal) -> Amount {
    USERS_BALANCES.with_borrow(|tag| tag.get(&user).unwrap_or_default())
}

fn update_user_balance(user: Principal, new_balance: u128) {
    USERS_BALANCES.with_borrow_mut(|reference| reference.insert(user, new_balance));
}

fn _initiate_scheduling_for_price_wait_operation(market_index: u64) -> TimerId {
    let current_timer_id = _get_market_timer_detail(market_index);
    ic_cdk_timers::clear_timer(current_timer_id);

    let new_timer_id = ic_cdk_timers::set_timer(Duration::from_secs(4), move || {
        ic_cdk::futures::spawn(schedule_execution_of_wait_operations(market_index));
    });
    new_timer_id
}

fn admin_guard() -> Result<(), String> {
    ADMIN.with_borrow(|admin_ref| {
        let admin = admin_ref.get().clone();
        if ic_cdk::api::msg_caller() == admin {
            return Ok(());
        } else {
            return Err("Invalid".to_string());
        };
    })
}

#[update(guard = "admin_guard")]
pub fn start_timer_for_market(market_index: u64) {
    ic_cdk_timers::set_timer_interval(Duration::from_secs(60), move || {
        collect_borrow_fees(market_index);
    });

    _set_market_timer_detail(market_index);
}

fn collect_borrow_fees(market_index: u64) {
    let outcome = _collect_borrow_fees(market_index);

    let last_time_updated = _get_market_timer_details(market_index);

    let hours_since_last_updated = duration_in_hours(last_time_updated);

    if hours_since_last_updated >= 8 {
        _collect_funding_fees(market_index)
    };

    if outcome == false {
        let current_timer_id = _get_market_timer_detail(market_index);
        ic_cdk_timers::clear_timer(current_timer_id);

        let new_timer_id = ic_cdk_timers::set_timer(Duration::from_secs(4), move || {
            ic_cdk::futures::spawn(schedule_execution_of_wait_operations(market_index));
        });

        _put_waiting_position(
            market_index,
            new_timer_id,
            PriceWaitingOperation::CollectBorrowingFeesOp,
            false,
        );
    }
}

async fn _update_price(market_index: u64) {
    let mut market = MARKETS.with_borrow(|reference| reference.get(market_index).unwrap());
    let quote_asset = _get_house_asset_pricing_details();
    let base_asset = market.index_asset_pricing_details();
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

fn duration_in_hours(start_time: u64) -> u64 {
    let duration_in_nano_secs = time() - start_time;

    return duration_in_nano_secs / ONE_HOUR_NANOSECONDS;
}

enum PriceWaitingOperation {
    ClosePositionOp(u128, Position),
    OpenPositionOp(u128, Position),
    MarketLiquidityOp {
        depositor: Principal,
        deposit: u128,
        min_amount_out: u128,
    },
    CollectBorrowingFeesOp,
}

export_candid!();

pub mod asset;
pub mod bias;
pub mod constants;
pub mod market;
pub mod math;
pub mod oracle;
pub mod position;
pub mod types;
pub mod vault;

#[cfg(test)]
pub mod unit_tests;

use ic_stable_structures::memory_manager::MemoryId;

/// Memeory locarions
pub const _ADMIN_MEMORY_ID: MemoryId = MemoryId::new(1);
pub const _HOUSE_DETAILS_MEMORY_ID: MemoryId = MemoryId::new(2);
pub const _MARKETS_MEMORY_ID: MemoryId = MemoryId::new(3);
pub const _BALANCES_MEMORY_ID: MemoryId = MemoryId::new(4);
pub const _MARKET_SHARE_USER_BALANCES_MEMORY_ID: MemoryId = MemoryId::new(5);
pub const _POSITIONS_MEMORY_ID: MemoryId = MemoryId::new(6);
pub const _MARKET_LIQUIDTY_SHARES_MEMORY_ID: MemoryId = MemoryId::new(7);

pub const COLLECT_BORROW_FEES_PRIORITY_INDEX: u8 = 0;
pub const ADD_LIQUIDITY_PRIORITY_INDEX: u8 = 1;
pub const CLOSE_POSITION_PRIORITY_INDEX: u8 = 2;
pub const OPEN_POSITION_PRIORITY_INDEX: u8 = 3;
pub const REMOVE_LIQUIDITY_PRIORITY_INDEX: u8 = 4;

pub const ONE_HOUR_NANOSECONDS: u64 = 60 * 60 * 1_000_000_000;
pub const _ONE_SECOND: u64 = 1_000_000_000;
pub const MAX_ALLOWED_PRICE_CHANGE_INTERVAL: u64 = 600_000_000_000; // 10 minutes 

// collect borow fees
// close positon
// add liquidity
// open position
// remove liquidity

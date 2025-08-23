use ic_stable_structures::memory_manager::MemoryId;

/// Memeory locarions
pub const _ADMIN_MEMORY: MemoryId = MemoryId::new(1);
pub const _HOUSE_DETAILS_MEMORY: MemoryId = MemoryId::new(2);
pub const _XRC_MEMORY: MemoryId = MemoryId::new(3);

pub const _MARKETS_MEMORY: MemoryId = MemoryId::new(4);
pub const _BALANCES_MEMORY: MemoryId = MemoryId::new(5);
pub const _POSITIONS_MEMORY: MemoryId = MemoryId::new(6);
pub const _MARKET_LIQUIDTY_SHARES_MEMORY: MemoryId = MemoryId::new(7);

pub const ONE_HOUR_NANOSECONDS: u64 = 60 * 60 * 1_000_000_000;
pub const _ONE_SECOND: u64 = 1_000_000_000;
pub const MAX_ALLOWED_PRICE_CHANGE_INTERVAL: u64 = 600_000_000_000;

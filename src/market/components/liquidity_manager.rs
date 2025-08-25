use candid::CandidType;
use serde::{Deserialize, Serialize};

#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
#[derive(Default, Deserialize, Copy, Clone, Serialize, CandidType)]
pub struct HouseLiquidityManager {
    pub total_liquidity_tokens_minted: u128,
    /// Total Deposit
    ///
    /// total deposit into a amrket by both liquidity providers and traders
    pub total_deposit: u128,
    /// Current Net Positions Reserve
    ///
    /// current net positions reserve
    pub current_longs_reserve: u128,
    /// Current Net Positions Reserve
    ///
    /// current net positions reserve
    pub current_shorts_reserve: u128,
    /// Current Net debt
    ///
    /// the total debt owed by both longs and shorts
    pub current_net_debt: u128,
    /// Bad Debt
    ///
    /// bad debt owed by house
    /// Bad debt occurrs when positons that shold be liquidated are not liquidated on time, the positions debt on funding fees given
    /// to the house fot the particular market ,  
    pub bad_debt: u128,
    /// Free Liquidity
    ///
    /// unsused house liquidity
    pub free_liquidity: u128,
    /// Current Borrow Fees Owed
    ///
    /// The current borrow fees collected for for all current opened positons
    /// This tracks the current borrow fees that is unpaid by current open positons
    pub current_borrow_fees_owed: u128,
    /// Longs Max Reserve Factor
    ///  
    ///
    pub longs_max_reserve_factor: u128,
    pub shorts_max_reserve_factor: u128,

    /// liquidation factor for  
    pub liquidation_factor: u128,
    pub last_time_since_borrow_fees_collected: u64,
}

impl HouseLiquidityManager {
    /// The House value is difference of the net sum of tokens in the pool (excluding losses or gains from traders positions)
    /// and the current bad debt of the pool
    /// @dev it is returned as a signed integer becasue in rare cases of extreme bad debt ,this value might be less than zero
    pub fn static_value(&self) -> i128 {
        (self.free_liquidity
            + self.current_longs_reserve
            + self.current_shorts_reserve
            + self.current_net_debt
            + self.current_borrow_fees_owed) as i128
            - self.bad_debt as i128
    }
}

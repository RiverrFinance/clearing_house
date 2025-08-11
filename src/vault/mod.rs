use crate::{
    math::math::{_ONE_PERCENT, _percentage},
    types::AssetPricingDetails,
};

const YEAR: Time = 31_536_000_000_000_000;

const MONTH: Time = 2_628_000_000_000_000;

type Amount = u128;
type Time = u64;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum LockSpan {
    Month2,
    Month6,
    Year,
}

impl Default for LockSpan {
    fn default() -> Self {
        return LockSpan::Month2;
    }
}

#[derive(Default)]
pub struct LockDetails {
    pub stake_span: LockSpan,
    pub amount: Amount,
    pub expiry_time: Time,
    pub pre_earnings: Amount,
}

/// Vault handles all vault activities

#[derive(Default)]
pub struct Vault {
    pub max_vault_utilization: u64,
    // // total amount borrowed as debt
    // pub debt: u128,
    // Amount still unutllised
    pub free_liquidity: Amount,
    // Note this does not track rewards for locking
    pub total_liquidity_in_vault: Amount,

    // total pool token minted
    pub total_vault_share_minted: Amount,

    /// Lockdetails
    pub span2_details: LockDurationDetails,
    pub span6_details: LockDurationDetails,
    pub span12_details: LockDurationDetails,
}

impl Vault {
    /// Create Stake function
    ///
    ///
    /// Params
    ///  - Amount :The amount of asset being put staked or deposited
    ///  - Current Lifetime Earnings :The total amount since first epoch of asset  received as fees to leverage provider from traders trading with leverage
    ///  - Stake Span :The specific staking duration
    ///
    /// Returns
    ///  - StakeDetails :The details of the newly created stake
    pub fn _create_lock(&mut self, amount: Amount, stake_span: LockSpan) -> LockDetails {
        let (span_lifetime_earnings_per_token, span_init_total_locked, expiry_time) =
            self._update_specific_span_details(amount, stake_span, true);

        let pre_earnings = if span_init_total_locked == 0 {
            0
        } else {
            (amount * span_lifetime_earnings_per_token) / base_units()
        };

        let stake_details = LockDetails {
            stake_span,
            amount,
            pre_earnings,
            expiry_time,
        };

        return stake_details;
    }

    /// Update Fees Across Span Function
    ///
    /// Updates the fees for all staking duration spans (instant, 2 months, 6 months, and 1 year)
    /// using the current lifetime fees accumulated in the vault.
    ///
    /// The function updates:
    /// - Instant staking span (span0) with no duration multiplier
    /// - 2 month staking span (span2) with 2x duration multiplier  
    /// - 6 month staking span (span6) with 6x duration multiplier
    /// - 12 month staking span (span12) with 12x duration multiplier
    pub fn _update_fees_across_span(&mut self, fee_earned: Amount) {
        self.span2_details._update_earnings(fee_earned, Some(2));
        self.span6_details._update_earnings(fee_earned, Some(6));
        self.span12_details._update_earnings(fee_earned, Some(12));
    }

    /// Calculate Stake Earnings Function
    ///
    /// Calculates the earnings for a given stake by determining the lifetime earnings per token
    /// for the stake's duration span and computing the user's share of earnings.
    ///
    /// Params
    /// - ref_stake: StakeDetails - The stake details containing amount, span, and pre-earnings information
    ///
    /// Returns
    /// - Amount - The total earnings for this stake (current earnings minus pre-earnings)
    pub fn _calc_lock_earnings(&self, ref_stake: LockDetails) -> Amount {
        let lifetime_earnings_per_token;
        match ref_stake.stake_span {
            LockSpan::Month2 => {
                lifetime_earnings_per_token = self.span2_details.lifetime_earnings_per_token
            }
            LockSpan::Month6 => {
                lifetime_earnings_per_token = self.span6_details.lifetime_earnings_per_token
            }
            LockSpan::Year => {
                lifetime_earnings_per_token = self.span12_details.lifetime_earnings_per_token
            }
        };

        let amount_earned = (ref_stake.amount * lifetime_earnings_per_token) / base_units();

        let user_earnings = amount_earned - ref_stake.pre_earnings;

        return user_earnings;
    }

    /// Close Stake Function
    ///
    /// Params
    ///  - Reference Stake :The stake details of the reference stake to close
    ///
    /// Returns
    ///  - Earnings :The amount earned by the particular stake for the entire staking duration
    pub fn _open_lock(&mut self, reference_stake: LockDetails) {
        match reference_stake.stake_span {
            LockSpan::Month2 => self
                .span2_details
                .update_total_locked(reference_stake.amount, false),
            LockSpan::Month6 => self
                .span6_details
                .update_total_locked(reference_stake.amount, false),
            LockSpan::Year => self
                .span12_details
                .update_total_locked(reference_stake.amount, false),
        };
    }

    /// Update Asset Staking Details Function

    /// # Params
    /// * `amount` - The amount of tokens being staked or unstaked
    /// * `specific_span` - The staking duration period (Instant, 2 Months, 6 Months, or 1 Year)
    /// * `lock` - Boolean indicating if tokens are being locked (true) or unlocked (false)
    ///
    /// # Returns
    /// A tuple containing:
    /// * The lifetime earnings per token for the specific stake duration
    /// * The total amount of tokens locked in this stake duration before this update
    /// * The expiry timestamp when these staked tokens can be withdrawn
    ///
    /// # Details
    /// This function handles updating staking details when tokens are staked or unstaked.
    /// For each stake duration, it:
    /// 1. Records the current total locked amount
    /// 2. Updates the staking details with the new amount
    /// 3. Calculates the expiry time based on the stake duration
    pub fn _update_specific_span_details(
        &mut self,
        amount: Amount,
        specific_span: LockSpan,
        lock: bool,
    ) -> (Amount, Amount, Time) {
        let span_lifetime_earnings_per_token;
        let span_init_total_locked;
        let expiry_time;

        match specific_span {
            LockSpan::Month2 => {
                span_init_total_locked = self.span2_details.total_locked;

                span_lifetime_earnings_per_token =
                    self.span2_details._lifetime_earnings_per_token();
                self.span2_details.update_total_locked(amount, lock);
                expiry_time = ic_cdk::api::time() + (2 * MONTH);
            }
            LockSpan::Month6 => {
                span_init_total_locked = self.span6_details.total_locked;

                span_lifetime_earnings_per_token =
                    self.span6_details._lifetime_earnings_per_token();
                self.span6_details.update_total_locked(amount, lock);
                expiry_time = ic_cdk::api::time() + (6 * MONTH)
            }
            LockSpan::Year => {
                span_init_total_locked = self.span12_details.total_locked;

                span_lifetime_earnings_per_token =
                    self.span12_details._lifetime_earnings_per_token();
                self.span12_details.update_total_locked(amount, lock);
                expiry_time = ic_cdk::api::time() + YEAR
            }
        }

        return (
            span_lifetime_earnings_per_token,
            span_init_total_locked,
            expiry_time,
        );
    }
}

pub struct LiquidityManagerDetails {
    pub asset: AssetPricingDetails,
    pub virtual_asset: AssetPricingDetails,
    pub min_amount: Amount,
}

#[derive(Default)]
pub struct LockDurationDetails {
    /// The total Amount earned by a single token since span creation
    pub lifetime_earnings_per_token: Amount,
    /// Total Locked
    ///
    /// The total Amount of liquidity locked in that particular span
    pub total_locked: Amount,
}

impl LockDurationDetails {
    pub fn _lifetime_earnings_per_token(&self) -> Amount {
        return self.lifetime_earnings_per_token;
    }
    /// Updates fees for a staking duration
    ///
    /// # Parameters
    /// - `current_all_time_earnings`: latest fees received
    /// - `span_share`: Optional share value for the staking duration

    pub fn _update_earnings(&mut self, fees_earned: Amount, span_share: Option<u128>) {
        let (percentage, share, total_share) = match span_share {
            Some(value) => (40 * _ONE_PERCENT, value, 20),
            None => (60 * _ONE_PERCENT, 1, 1),
        };

        let init_total_locked = if self.total_locked == 0 {
            1
        } else {
            self.total_locked
        };

        // new earnings
        let locked_new_earnings = _percentage(percentage, fees_earned);

        let span_new_earnings_per_token =
            (locked_new_earnings * share * base_units()) / (total_share * init_total_locked);

        self.lifetime_earnings_per_token += span_new_earnings_per_token;
    }
    /// Updates stake duration details by modifying the total locked amount
    ///
    /// # Parameters
    /// - `amount`: Amount to add or remove from stake duration
    /// - `lock`: If true, adds amount. If false, removes amount
    ///
    /// # Returns
    /// The lifetime earnings per staked token
    pub fn update_total_locked(&mut self, amount: Amount, lock: bool) {
        if lock {
            self.total_locked += amount
        } else {
            self.total_locked -= amount
        };
    }
}

fn base_units() -> Amount {
    10u128.pow(12)
}

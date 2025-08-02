use std::borrow::Cow;

use bincode::{Decode, Encode};
use ic_stable_structures::{Storable, storable::Bound};

use crate::{
    math::{_ONE_PERCENT, _percentage},
    types::Asset,
};

const YEAR: Time = 31_536_000_000_000_000;

const MONTH: Time = 2_628_000_000_000_000;

type Amount = u128;
type Time = u64;

#[derive(Copy, Clone, Decode, Encode, PartialEq, Eq)]
pub enum LockSpan {
    Instant,
    Month2,
    Month6,
    Year,
}

impl Default for LockSpan {
    fn default() -> Self {
        return LockSpan::Instant;
    }
}

#[derive(Encode, Decode, Default)]
pub struct LockDetails {
    pub stake_span: LockSpan,
    pub amount: Amount,
    pub expiry_time: Time,
    pub pre_earnings: Amount,
}

#[derive(Encode, Default, Decode)]
pub struct Vault {
    pub max_vault_utilization: u64,
    pub debt: u128,
    pub free_liquidity: Amount,
    // Note this does not track rewards for locking
    pub total_vault_value: Amount,
    pub total_vault_share_minted: Amount,
    pub span0_details: LockDurationDetails,
    pub span2_details: LockDurationDetails,
    pub span6_details: LockDurationDetails,
    pub span12_details: LockDurationDetails,
}

impl Vault {
    pub fn _utilization_rate() {}
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
        self.span0_details._update_earnings(fee_earned, None);
        self.span2_details._update_earnings(fee_earned, Some(2));
        self.span6_details._update_earnings(fee_earned, Some(6));
        self.span0_details._update_earnings(fee_earned, Some(12));
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
            LockSpan::Instant => {
                lifetime_earnings_per_token = self.span0_details.lifetime_earnings_per_token
            }
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
            LockSpan::Instant => self
                .span0_details
                .update_total_locked(reference_stake.amount, false),
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
            LockSpan::Instant => {
                span_init_total_locked = self.span0_details.total_locked;

                span_lifetime_earnings_per_token =
                    self.span0_details._lifetime_earnings_per_token();
                self.span0_details.update_total_locked(amount, lock);
                expiry_time = ic_cdk::api::time()
            }
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

impl Storable for Vault {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let mut slice = [0u8; 500];
        let length =
            bincode::encode_into_slice(self, &mut slice, bincode::config::standard()).unwrap();

        let slice = &slice[..length];
        Cow::Owned(slice.to_vec())
    }

    /// Converts the element into an owned byte vector.
    ///
    /// This method consumes `self` and avoids cloning when possible.
    fn into_bytes(self) -> Vec<u8> {
        let mut slice = [0u8; 500];
        let length =
            bincode::encode_into_slice(self, &mut slice, bincode::config::standard()).unwrap();

        let slice = &slice[..length];
        slice.to_vec()
    }

    /// Converts bytes into an element.
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        bincode::decode_from_slice(bytes.as_ref(), bincode::config::standard())
            .expect("Failed to decode MarketDetails")
            .0
    }

    /// The size bounds of the type.
    const BOUND: Bound = Bound::Unbounded;

    /// Like `to_bytes`, but checks that bytes conform to declared bounds.
    fn to_bytes_checked(&self) -> Cow<'_, [u8]> {
        let bytes = self.to_bytes();
        Self::check_bounds(&bytes);
        bytes
    }

    /// Like `into_bytes`, but checks that bytes conform to declared bounds.
    fn into_bytes_checked(self) -> Vec<u8>
    where
        Self: Sized,
    {
        let bytes = self.into_bytes();
        Self::check_bounds(&bytes);
        bytes
    }

    #[inline]
    fn check_bounds(bytes: &[u8]) {
        if let Bound::Bounded {
            max_size,
            is_fixed_size,
        } = Self::BOUND
        {
            let actual = bytes.len();
            if is_fixed_size {
                assert_eq!(
                    actual, max_size as usize,
                    "expected a fixed-size element with length {} bytes, but found {} bytes",
                    max_size, actual
                );
            } else {
                assert!(
                    actual <= max_size as usize,
                    "expected an element with length <= {} bytes, but found {} bytes",
                    max_size,
                    actual
                );
            }
        }
    }
}

pub struct LiquidityManagerDetails {
    pub asset: Asset,
    pub virtual_asset: Asset,
    pub min_amount: Amount,
}

#[derive(Encode, Default, Decode)]
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

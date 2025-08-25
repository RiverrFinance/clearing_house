use std::borrow::Cow;

use candid::CandidType;
use ic_stable_structures::{Storable, storable::Bound};
use serde::{Deserialize, Serialize};

use crate::asset_management::AssetLedger;
use crate::pricing_update_management::price_fetch::AssetPricingDetails;
use crate::stable_memory::HOUSE_SETTINGS;

#[derive(Serialize, Deserialize, CandidType, Clone)]
pub struct HouseDetails {
    pub house_asset_ledger: AssetLedger,
    pub house_asset_pricing_details: AssetPricingDetails,
    pub execution_fee: u128,
}

pub fn get_house_asset_ledger() -> AssetLedger {
    HOUSE_SETTINGS.with_borrow(|reference| reference.get().house_asset_ledger)
}

pub fn get_execution_fee() -> u128 {
    HOUSE_SETTINGS.with_borrow(|reference| reference.get().execution_fee)
}

// pub fn get_markets_tokens_ledger() -> AssetLedger {
//     HOUSE_SETTINGS.with_borrow(|reference| reference.get().markets_tokens_ledger)
// }

pub fn get_house_asset_pricing_details() -> AssetPricingDetails {
    HOUSE_SETTINGS.with_borrow(|reference| reference.get().house_asset_pricing_details.clone())
}

impl Default for HouseDetails {
    fn default() -> Self {
        Self {
            // markets_tokens_ledger: AssetLedger::default(),
            house_asset_pricing_details: AssetPricingDetails::default(),
            execution_fee: 0,
            house_asset_ledger: AssetLedger::default(),
        }
    }
}

impl Storable for HouseDetails {
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

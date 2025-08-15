use std::borrow::Cow;

use candid::{CandidType, Principal};
use ic_cdk::call::Call;

use ic_stable_structures::{Storable, storable::Bound};
use serde::{Deserialize, Serialize};

use crate::asset::{AssetLedger, AssetPricingDetails};

pub type Amount = u128;
pub type Time = u64;

#[derive(Serialize, Deserialize, CandidType, Clone)]
pub struct HouseDetails {
    pub house_asset_ledger: AssetLedger,
    pub house_asset_pricing_details: AssetPricingDetails,
    pub markets_tokens_ledger: AssetLedger,
    pub execution_fee: u128,
    pub execution_fee_collected: u128,
}

impl Default for HouseDetails {
    fn default() -> Self {
        Self {
            markets_tokens_ledger: AssetLedger::default(),
            house_asset_pricing_details: AssetPricingDetails::default(),
            execution_fee: 0,
            house_asset_ledger: AssetLedger::default(),
            execution_fee_collected: 0,
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

#[derive(CandidType, Deserialize)]
pub struct GetExchangeRateRequest {
    /// The base asset, i.e., the first asset in a currency pair. For example,
    /// ICP is the base asset in the currency pair ICP/USD.
    pub base_asset: AssetPricingDetails,
    /// The quote asset, i.e., the second asset in a currency pair. For example,
    /// USD is the quote asset in the currency pair ICP/USD.
    pub quote_asset: AssetPricingDetails,
    /// An optional parameter used to find a rate at a specific time.
    pub timestamp: Option<u64>,
}

#[derive(CandidType, Default, Deserialize)]
pub struct ExchangeRateMetadata {
    /// The scaling factor for the exchange rate and the standard deviation.
    pub decimals: u32,
    /// The number of queried exchanges for the base asset.
    pub base_asset_num_queried_sources: usize,
    /// The number of rates successfully received from the queried sources for the base asset.
    pub base_asset_num_received_rates: usize,
    /// The number of queried exchanges for the quote asset.
    pub quote_asset_num_queried_sources: usize,
    /// The number of rates successfully received from the queried sources for the quote asset.
    pub quote_asset_num_received_rates: usize,
    /// The standard deviation of the received rates, scaled by the factor `10^decimals`.
    pub standard_deviation: u64,
    /// The timestamp of the beginning of the day for which the forex rates were retrieved, if any.
    pub forex_timestamp: Option<u64>,
}

#[derive(CandidType, Default, Deserialize)]
pub struct ExchangeRate {
    /// The base asset.
    pub base_asset: AssetPricingDetails,
    /// The quote asset.
    pub quote_asset: AssetPricingDetails,
    /// The timestamp associated with the returned rate.
    pub timestamp: u64,
    /// The median rate from the received rates, scaled by the factor `10^decimals` in the metadata.
    pub rate: u64,
    /// Metadata providing additional information about the exchange rate calculation.
    pub metadata: ExchangeRateMetadata,
}

#[derive(CandidType, Deserialize)]
pub enum ExchangeRateError {
    /// Returned when the canister receives a call from the anonymous principal.
    AnonymousPrincipalNotAllowed,
    /// Returned when the canister is in process of retrieving a rate from an exchange.
    Pending,
    /// Returned when the base asset rates are not found from the exchanges HTTP outcalls.
    CryptoBaseAssetNotFound,
    /// Returned when the quote asset rates are not found from the exchanges HTTP outcalls.
    CryptoQuoteAssetNotFound,
    /// Returned when the stablecoin rates are not found from the exchanges HTTP outcalls needed for computing a crypto/fiat pair.
    StablecoinRateNotFound,
    /// Returned when there are not enough stablecoin rates to determine the forex/USDT rate.
    StablecoinRateTooFewRates,
    /// Returned when the stablecoin rate is zero.
    StablecoinRateZeroRate,
    /// Returned when a rate for the provided forex asset could not be found at the provided timestamp.
    ForexInvalidTimestamp,
    /// Returned when the forex base asset is found.
    ForexBaseAssetNotFound,
    /// Returned when the forex quote asset is found.
    ForexQuoteAssetNotFound,
    /// Returned when neither forex asset is found.
    ForexAssetsNotFound,
    /// Returned when the caller is not the CMC and there are too many active requests.
    RateLimited,
    /// Returned when the caller does not send enough cycles to make a request.
    NotEnoughCycles,
    /// Returned if too many collected rates deviate substantially.
    InconsistentRatesReceived,
    /// Until candid bug is fixed, new errors after launch will be placed here.
    Other(OtherError),
}

#[derive(CandidType, Deserialize)]
pub struct OtherError {
    /// The identifier for the error that occurred.
    pub code: u32,
    /// A description of the error that occurred.
    pub description: String,
}

pub type GetExchangeRateResult = Result<ExchangeRate, ExchangeRateError>;

pub struct XRC {
    pub canister_id: Principal,
}

impl XRC {
    pub fn init(canister_id: Principal) -> Self {
        XRC { canister_id }
    }

    /// tries to fetch the current exchange rate of the pair and returns the result
    async fn _get_exchange_rate(&self, request: GetExchangeRateRequest) -> GetExchangeRateResult {
        let call = Call::unbounded_wait(self.canister_id, "get_exchange_rate")
            .with_arg(request)
            .with_cycles(1_000_000_000);

        return call.await.unwrap().candid().unwrap();
    }
}

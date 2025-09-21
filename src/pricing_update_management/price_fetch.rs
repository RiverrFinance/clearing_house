use std::str::FromStr;

use candid::{CandidType, Principal};

use ic_cdk::call::Call;
use serde::{Deserialize, Serialize};

use crate::house_settings::get_house_asset_pricing_details;
use crate::stable_memory::MARKETS_LIST;

const XRC_ID: &str = "uf6dk-hyaaa-aaaaq-qaaaq-cai";

pub async fn update_price(market_index: u64) {
    let mut market = MARKETS_LIST.with_borrow(|reference| reference.get(market_index).unwrap());
    let quote_asset = get_house_asset_pricing_details();
    let base_asset = market.index_asset_pricing_details();

    let request = GetExchangeRateRequest {
        base_asset,
        quote_asset,
        timestamp: None,
    };

    let result: GetExchangeRateResult = _get_exchange_rate(request).await;
    if let Ok(response) = result {
        market._update_price(response.rate, response.metadata.decimals);
        //  last_price_update_timer = time();
        MARKETS_LIST.with_borrow_mut(|reference| {
            reference.set(market_index, &market);
        });
    }
}

/// tries to fetch the current exchange rate of the pair and returns the result
pub async fn _get_exchange_rate(request: GetExchangeRateRequest) -> GetExchangeRateResult {
    let canister_id = Principal::from_str(XRC_ID).unwrap();
    let call = Call::unbounded_wait(canister_id, "get_exchange_rate")
        .with_arg(request)
        .with_cycles(1_000_000_000); // i trillion

    return call.await.unwrap().candid().unwrap();
}

#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
#[derive(Deserialize, Serialize, Copy, Clone, CandidType)]
pub enum AssetClass {
    /// The cryptocurrency asset class.
    Cryptocurrency,
    /// The fiat currency asset class.
    FiatCurrency,
}

impl Default for AssetClass {
    fn default() -> Self {
        AssetClass::Cryptocurrency
    }
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

#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
#[derive(Clone, Serialize, Deserialize, Default, CandidType)]
pub struct AssetPricingDetails {
    /// The symbol/code of the asset.
    pub symbol: String,
    /// The asset class.
    pub class: AssetClass,
}

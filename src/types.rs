use super::math::{_ONE_PERCENT, _calc_shares, _calc_shares_value};

pub type Amount = u128;
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

pub struct Asset {
    /// The symbol/code of the asset.
    pub symbol: String,
    /// The asset class.
    pub class: AssetClass,
}

pub struct GetExchangeRateRequest {
    /// The base asset, i.e., the first asset in a currency pair. For example,
    /// ICP is the base asset in the currency pair ICP/USD.
    pub base_asset: Asset,
    /// The quote asset, i.e., the second asset in a currency pair. For example,
    /// USD is the quote asset in the currency pair ICP/USD.
    pub quote_asset: Asset,
    /// An optional parameter used to find a rate at a specific time.
    pub timestamp: Option<u64>,
}

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

pub struct ExchangeRate {
    /// The base asset.
    pub base_asset: Asset,
    /// The quote asset.
    pub quote_asset: Asset,
    /// The timestamp associated with the returned rate.
    pub timestamp: u64,
    /// The median rate from the received rates, scaled by the factor `10^decimals` in the metadata.
    pub rate: u64,
    /// Metadata providing additional information about the exchange rate calculation.
    pub metadata: ExchangeRateMetadata,
}

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

pub struct OtherError {
    /// The identifier for the error that occurred.
    pub code: u32,
    /// A description of the error that occurred.
    pub description: String,
}

pub type GetExchangeRateResult = Result<ExchangeRate, ExchangeRateError>;

pub struct BiasDetails {
    traders_net_volume: u128,
    total_shares: u128,
    house_volume: i128,
    house_units: i128,
}

impl BiasDetails {
    pub fn add_volume(&mut self, delta: u128) -> Amount {
        let volume_share = _calc_shares(delta, self.total_shares, self.traders_net_volume);
        self.total_shares += volume_share;
        self.traders_net_volume += delta;
        return volume_share;
    }

    pub fn remove_volume(&mut self, delta: Amount) -> Amount {
        let value = _calc_shares_value(delta, self.total_shares, self.traders_net_volume);
        self.traders_net_volume -= value;
        self.total_shares -= delta;
        return value;
    }

    pub fn update_house_position(&mut self, delta_house_volume: i128, delta_units: i128) {
        self.house_volume += delta_house_volume;
        self.house_units += delta_units
    }
}

pub struct BiasTracker {
    pub long: BiasDetails,
    pub short: BiasDetails,
}

impl BiasTracker {
    pub fn add_volume(&mut self, delta: Amount, long: bool) -> Amount {
        if long {
            self.long.add_volume(delta)
        } else {
            self.short.add_volume(delta)
        }
    }

    pub fn remove_volume(&mut self, delta: Amount, long: bool) -> Amount {
        if long {
            self.long.remove_volume(delta)
        } else {
            self.short.remove_volume(delta)
        }
    }
}

pub fn _percentage<T>(x: u64, value: T) -> T
where
    T: std::ops::Mul<Output = T> + std::ops::Div<Output = T> + From<u64>,
{
    ((T::from(x)) * value) / T::from(100 * _ONE_PERCENT)
}

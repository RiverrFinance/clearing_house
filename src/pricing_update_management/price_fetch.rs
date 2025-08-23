// async fn _update_price(market_index: u64) {
//     let mut market = MARKETS.with_borrow(|reference| reference.get(market_index).unwrap());
//     let HouseDetails {
//         house_asset_pricing_details: quote_asset,
//         ..
//     } = _get_house_details();
//     let base_asset = market.index_asset_pricing_details();
//     let xrc_canister_id = _get_xrc_id();
//     let xrc = XRC::init(xrc_canister_id);
//     let request = GetExchangeRateRequest {
//         base_asset,
//         quote_asset,
//         timestamp: None,
//     };

//     let result: GetExchangeRateResult = xrc._get_exchange_rate(request).await;
//     if let Ok(response) = result {
//         market._update_price(response.rate, response.metadata.decimals);
//     }
// }

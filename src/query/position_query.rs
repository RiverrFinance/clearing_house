use candid::{CandidType, Principal};
use ic_cdk::query;
use serde::Deserialize;

use crate::{
    house_settings::get_house_asset_pricing_details,
    market::market_details::MarketDetails,
    math::math::{apply_precision, to_precision},
    position::position_details::PositionDetails,
    stable_memory::{MARKETS_LIST, USERS_POSITIONS},
};

#[derive(CandidType, Deserialize)]
pub struct QueryPositionDetailsResult {
    #[serde(rename = "positionId")]
    position_id: u64,
    #[serde(rename = "positionCurrentDetails")]
    position_current_details: GetPositionCurrentDetails,
    // size is in units
}

#[query(name = "getAllUserPositionsInMarket")]
pub fn get_all_user_positions_in_market(
    user: Principal,
    market_index: u64,
) -> (String, String, Vec<QueryPositionDetailsResult>) {
    let mut positions: Vec<QueryPositionDetailsResult> = Vec::new();

    let market_details = get_market_details(market_index);

    let quote_asset_symbol = get_house_asset_pricing_details().symbol.clone();
    let base_asset_symbol = market_details.index_asset_pricing_details.symbol.clone();

    USERS_POSITIONS.with_borrow(|positions_reference| {
        for entry in positions_reference.iter() {
            let ((owner, position_id), (index, position)) = (entry.key(), entry.value());

            if owner == &user && market_index == index {
                let current_cummulative_funding_factor =
                    market_details.get_cummulative_funding_factor_since_epoch(position.long);
                let current_cummulative_borrowing_factor =
                    market_details.get_cummulative_borrowing_factor_since_epoch(position.long);
                let liquidation_factor = market_details.liquidity_state.liquidation_factor;
                let position_current_details = get_position_current_details(
                    position,
                    current_cummulative_funding_factor,
                    current_cummulative_borrowing_factor,
                    liquidation_factor,
                );
                positions.push(QueryPositionDetailsResult {
                    position_id: *position_id,
                    position_current_details,
                });
            }
        }
    });

    (base_asset_symbol, quote_asset_symbol, positions)
}

fn get_market_details(market_id: u64) -> MarketDetails {
    MARKETS_LIST.with_borrow(|market_reference| {
        let market = market_reference.get(market_id).unwrap();
        market
    })
}

#[derive(CandidType, Deserialize)]
pub struct GetPositionCurrentDetails {
    #[serde(rename = "isLong")]
    is_long: bool,
    #[serde(rename = "currentCollateral")]
    current_collateral: u128,
    #[serde(rename = "netBorrowingFee")]
    net_borrowing_fee: u128,
    #[serde(rename = "netFundingFee")]
    net_funding_fee: i128,
    #[serde(rename = "liquidationPrice")]
    liquidation_price: u128,
    #[serde(rename = "positionSize")]
    position_size: u128,
}

fn get_position_current_details(
    position: PositionDetails,
    current_cummulative_funding_factor: i128,
    current_cummulative_borrowing_factor: u128,
    liquidation_factor: u128,
) -> GetPositionCurrentDetails {
    let net_funding_fee = position.get_net_funding_fee(current_cummulative_funding_factor);
    let net_borrowing_fee = position.get_net_borrowing_fee(current_cummulative_borrowing_factor);

    let current_collateral = i128::max(
        position.collateral as i128 + net_funding_fee - net_borrowing_fee as i128,
        0,
    ) as u128;

    let liquidation_price = _get_liquidation_price(
        current_collateral,
        position.debt,
        position.units,
        liquidation_factor,
        position.long,
    );

    GetPositionCurrentDetails {
        current_collateral,
        net_borrowing_fee,
        net_funding_fee,
        liquidation_price,
        is_long: position.long,
        position_size: position.units,
    }
}

fn _get_liquidation_price(
    current_collateral: u128,
    position_debt: u128,
    position_units: u128,
    liquidation_factor: u128,
    is_long: bool,
) -> u128 {
    // the maximum possible loss due to pnl before the next borrow of funding fee settlement
    let max_instant_pnl_loss =
        current_collateral - apply_precision(liquidation_factor, current_collateral);

    let liquidation_price = if is_long {
        // For long positions: price decreases as position loses value
        // Entry value = collateral + debt
        let current_position_value = current_collateral + position_debt;

        // Liquidation occurs when position value drops by max_loss_allowed
        let liquidation_value = current_position_value - max_instant_pnl_loss;

        to_precision(liquidation_value, position_units)
    } else {
        // For short positions: price increases as position loses value
        // Entry value = collateral + debt
        let entry_value = current_collateral + position_debt;

        // Liquidation occurs when position value increases by max_loss_allowed
        let liquidation_value = entry_value + max_instant_pnl_loss;

        to_precision(liquidation_value, position_units)
    };

    liquidation_price
}

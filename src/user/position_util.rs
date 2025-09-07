use candid::Principal;

use crate::{
    math::math::{apply_precision, to_precision},
    position::position_details::PositionDetails,
    stable_memory::{MARKETS_WITH_LAST_PRICE_UPDATE_TIME, USERS_POSITIONS},
};

#[derive(Debug, Clone)]
pub struct QueryGetUserPositionState {
    pub borrowing_fees_owned: u128,
    pub funding_fees_pay: u128,
    pub liquidation_price: u128,
}

pub fn _get_user_position_details(user: Principal, position_id: u64) -> (u64, PositionDetails) {
    USERS_POSITIONS.with_borrow(|reference| reference.get(&(user, position_id)).unwrap())
}

pub fn _put_user_position_detail(
    user: Principal,
    market_index: u64,
    position_id: u64,
    position_details: PositionDetails,
) {
    USERS_POSITIONS.with_borrow_mut(|reference| {
        reference.insert((user, position_id), (market_index, position_details));
    });
}

pub fn remove_user_position_detail(user: Principal, position_id: u64) {
    USERS_POSITIONS.with_borrow_mut(|reference| {
        reference.remove(&(user, position_id));
    });
}

pub fn try_get_user_position_details(
    user: Principal,
    position_id: u64,
) -> Option<(u64, PositionDetails)> {
    USERS_POSITIONS.with_borrow(|reference| reference.get(&(user, position_id)))
}

/// A liquidation price of 0 indicates the position is already liquidatable.

/// Calculate the liquidation price for a position based on current amount owed
///
/// # Arguments
/// * `user` - The user principal
/// * `position_id` - The position ID
/// * `market_price` - Current market price (optional, used for validation)
///
/// # Returns
/// * `Option<u128>` - The liquidation price, or None if position doesn't exist or calculation fails
pub fn calculate_liquidation_price(user: Principal, position_id: u64) -> Option<u128> {
    // Get position details
    let (market_index, position_details) = match try_get_user_position_details(user, position_id) {
        Some(details) => details,
        None => return None,
    };

    // Get market details
    let market = MARKETS_WITH_LAST_PRICE_UPDATE_TIME
        .with_borrow(|reference| reference.get(market_index).map(|(market, _)| market))?;

    // Get current cumulative factors
    let current_cummulative_funding_factor =
        market.get_cummulative_funding_factor_since_epoch(position_details.long);
    let current_cummulative_borrowing_factor =
        market.get_cummulative_borrowing_factor_since_epoch(position_details.long);

    // Calculate current fees owed
    let net_borrowing_fee =
        position_details.get_net_borrowing_fee(current_cummulative_borrowing_factor);
    let net_funding_fee = position_details.get_net_funding_fee(current_cummulative_funding_factor);

    // Get liquidation factor from market
    let liquidation_factor = market.liquidity_manager.liquidation_factor;

    // Calculate the minimum collateral required to avoid liquidation
    let min_collateral_required = apply_precision(liquidation_factor, position_details.collateral);

    let adjusted_collateral = i128::max(
        position_details.collateral as i128 + net_funding_fee - net_borrowing_fee as i128,
        0,
    ) as u128;

    // If adjusted collateral is already below minimum required, position is liquidatable at any price
    if adjusted_collateral < min_collateral_required {
        return Some(0); // Position is already liquidatable
    }

    // Calculate the maximum loss the position can sustain before liquidation
    let max_loss_allowed = adjusted_collateral - min_collateral_required;

    // Calculate liquidation price based on position direction and units
    if position_details.units == 0 {
        return None; // Invalid position with zero units
    }

    let liquidation_price = if position_details.long {
        // For long positions: price decreases as position loses value
        // Entry value = collateral + debt
        let entry_value = position_details.collateral + position_details.debt;

        // Liquidation occurs when position value drops by max_loss_allowed
        let liquidation_value = if entry_value > max_loss_allowed {
            entry_value - max_loss_allowed
        } else {
            0
        };

        to_precision(liquidation_value, position_details.units)
    } else {
        // For short positions: price increases as position loses value
        // Entry value = collateral + debt
        let entry_value = position_details.collateral + position_details.debt;

        // Liquidation occurs when position value increases by max_loss_allowed
        let liquidation_value = entry_value + max_loss_allowed;

        to_precision(liquidation_value, position_details.units)
    };

    Some(liquidation_price)
}

/// Get comprehensive position state including liquidation price
///
/// # Arguments
/// * `user` - The user principal
/// * `position_id` - The position ID
/// * `market_price` - Current market price
///
/// # Returns
/// * `Option<QueryGetUserPositionState>` - Position state with liquidation price
pub fn get_position_state_with_liquidation_price(
    user: Principal,
    position_id: u64,
) -> Option<QueryGetUserPositionState> {
    let (market_index, position_details) = match try_get_user_position_details(user, position_id) {
        Some(details) => details,
        None => return None,
    };

    let market = MARKETS_WITH_LAST_PRICE_UPDATE_TIME
        .with_borrow(|reference| reference.get(market_index).map(|(market, _)| market))?;

    let current_cummulative_funding_factor =
        market.get_cummulative_funding_factor_since_epoch(position_details.long);
    let current_cummulative_borrowing_factor =
        market.get_cummulative_borrowing_factor_since_epoch(position_details.long);

    let net_borrowing_fee =
        position_details.get_net_borrowing_fee(current_cummulative_borrowing_factor);
    let net_funding_fee = position_details.get_net_funding_fee(current_cummulative_funding_factor);

    let liquidation_price = calculate_liquidation_price(user, position_id)?;

    Some(QueryGetUserPositionState {
        borrowing_fees_owned: net_borrowing_fee,
        funding_fees_pay: net_funding_fee.abs() as u128,
        liquidation_price,
    })
}

/// Calculate the current margin ratio for a position
///
/// # Arguments
/// * `user` - The user principal
/// * `position_id` - The position ID
/// * `market_price` - Current market price
///
/// # Returns
/// * `Option<f64>` - The margin ratio (0.0 to 1.0), or None if position doesn't exist
pub fn calculate_margin_ratio(
    user: Principal,
    position_id: u64,
    market_price: u128,
) -> Option<f64> {
    let (market_index, position_details) = match try_get_user_position_details(user, position_id) {
        Some(details) => details,
        None => return None,
    };

    let market = MARKETS_WITH_LAST_PRICE_UPDATE_TIME
        .with_borrow(|reference| reference.get(market_index).map(|(market, _)| market))?;

    // Get current fees
    let current_cummulative_funding_factor =
        market.get_cummulative_funding_factor_since_epoch(position_details.long);
    let current_cummulative_borrowing_factor =
        market.get_cummulative_borrowing_factor_since_epoch(position_details.long);

    let net_borrowing_fee =
        position_details.get_net_borrowing_fee(current_cummulative_borrowing_factor);
    let net_funding_fee = position_details.get_net_funding_fee(current_cummulative_funding_factor);

    // Calculate current PnL
    let current_pnl = position_details.get_pnl(market_price);

    // Calculate effective collateral
    let effective_collateral = if position_details.collateral > net_borrowing_fee {
        position_details.collateral - net_borrowing_fee
    } else {
        0
    };

    let adjusted_collateral = effective_collateral
        + if net_funding_fee > 0 {
            net_funding_fee as u128
        } else {
            0
        };

    // Calculate current position value
    let current_position_value = if current_pnl > 0 {
        adjusted_collateral + current_pnl as u128
    } else {
        if adjusted_collateral > current_pnl.abs() as u128 {
            adjusted_collateral - current_pnl.abs() as u128
        } else {
            0
        }
    };

    // Calculate margin ratio
    let liquidation_factor = market.liquidity_manager.liquidation_factor;
    let min_collateral_required = apply_precision(liquidation_factor, position_details.collateral);

    if min_collateral_required == 0 {
        return Some(1.0); // No liquidation threshold
    }

    let margin_ratio = current_position_value as f64 / min_collateral_required as f64;
    Some(margin_ratio.min(1.0).max(0.0))
}

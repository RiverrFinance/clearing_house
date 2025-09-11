use crate::{math::math::bound_below_signed, position::position_details::PositionDetails};

/// ClosePositon With  Net Positive Funding
///
/// and  Close Position with Net Negative Funding ( see below)
///
/// Params
///
/// Free Liquidity - the amount of free liquidity
/// Net Funding - magnitude of net  funding fee to be received or paid  by position
/// Net borrowing Fee - net booring fee to be paid by position
/// Position PNL - the profit or loss of that position
///
/// Returns
///
/// Net free liquidity,
/// Collateral Out ( this incliudes both the collateral and the profit made) ,
/// House Debt ,in extreme cases of where a position can not fully pay its funding fee ,
/// this is repaid by the house and using its free liquidity and if free liquidity is not enough a bad debt occurs
pub fn close_position_with_net_positive_funding(
    position: PositionDetails,
    free_liquidity: u128,
    net_funding_fee_magnitude: i128,
    net_borrowing_fee: u128,
    position_pnl: i128,
) -> (u128, u128, u128) {
    let PositionDetails {
        debt, max_reserve, ..
    } = position;
    let position_pnl_magnitude = position_pnl.abs();
    let open_interest = position.open_interest();

    let mut net_free_liquidity = free_liquidity;

    let net_position_value = bound_below_signed(
        open_interest as i128 + position_pnl + net_funding_fee_magnitude
            - (net_borrowing_fee + debt) as i128,
        0,
    );

    if position_pnl.is_negative() {
        // trader losses ,house gains
        let position_value_without_pnl = bound_below_signed(
            open_interest as i128 - (net_borrowing_fee + debt) as i128 + net_funding_fee_magnitude,
            0,
        );

        let house_profit_from_negative_position_pnl =
            position_value_without_pnl.min(position_pnl_magnitude);

        //
        net_free_liquidity += max_reserve + house_profit_from_negative_position_pnl as u128;
    } else {
        // trader gains , house losses
        net_free_liquidity += max_reserve - position_pnl_magnitude as u128
    }

    return (net_free_liquidity, net_position_value as u128, 0);
}

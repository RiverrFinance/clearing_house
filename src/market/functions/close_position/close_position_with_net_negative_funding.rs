use crate::{math::math::bound_below_signed, position::position_details::PositionDetails};

/// Returns  (net_free_liquidity, funding_paid, collateral_out,bad_debt)
pub fn close_position_with_net_negative_funding(
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

    let position_value_without_funding_pay = bound_below_signed(
        open_interest as i128 + position_pnl - (net_borrowing_fee + debt) as i128,
        0,
    );

    let mut collateral_out = 0;

    let mut bad_debt = 0;

    if position_value_without_funding_pay >= net_funding_fee_magnitude {
        // if position_value after debt is enough to pay funding fee
        // collateral is what is left
        collateral_out = (position_value_without_funding_pay - net_funding_fee_magnitude) as u128;

        // net free liquidity is max_reserve - position_pnl
        net_free_liquidity += (max_reserve as i128 - position_pnl) as u128
    } else {
        // This block of code  tracks extreme cases like
        // when position funding_fee_can not be paid fully by a position
        // thsi incurs bad debt for the house
        if position_pnl.is_negative() {
            let position_value_without_pnl = bound_below_signed(
                open_interest as i128
                    - (net_borrowing_fee + debt) as i128
                    - net_funding_fee_magnitude,
                0,
            );

            // amount extractable to house
            let house_profit_from_negative_position_pnl =
                position_value_without_pnl.min(position_pnl_magnitude);
            //
            //
            net_free_liquidity += max_reserve + house_profit_from_negative_position_pnl as u128;
        } else {
            net_free_liquidity += (max_reserve as i128 - position_pnl_magnitude) as u128
        }

        let delta = net_free_liquidity as i128 + position_value_without_funding_pay // basically free liquidity + anything remainng in position - net funding fee magnitude
                    - net_funding_fee_magnitude;

        if delta >= 0 {
            net_free_liquidity = delta as u128;
        } else {
            net_free_liquidity = 0;
            bad_debt = delta.abs() as u128;
            //funding_paid = (delta + net_funding_fee_magnitude) as u128
        }
    }

    return (net_free_liquidity, collateral_out, bad_debt);
}

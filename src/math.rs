type Amount = u128;

pub const _ONE_PERCENT: u64 = 100_000;

pub fn _calc_shares(
    amount_in: Amount,
    init_total_shares: Amount,
    init_liquidity: Amount,
) -> Amount {
    if init_total_shares == 0 {
        return amount_in;
    }
    // unsafe
    return (amount_in * init_total_shares) / init_liquidity;
}

/// This function calculates the value of a particular share given the current  amount of shares  and the  current net liquidity
pub fn _calc_shares_value(
    shares: Amount,
    init_total_shares: Amount,
    init_liquidity: Amount,
) -> Amount {
    // unsafe
    return (shares * init_liquidity) / init_total_shares;
}

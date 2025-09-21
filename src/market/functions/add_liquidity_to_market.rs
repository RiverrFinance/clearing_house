use crate::{
    market::market_details::{LiquidityOperationResult, MarketDetails},
    math::math::mul_div,
};

use crate::market::components::liquidity_state::HouseLiquidityState;

pub struct AddLiquidityToMarketParams {
    /// The amount of quote asset to deposit into the market's liquidity pool.
    /// This should be specified with 20 decimal places precision (e.g., 10000000000000000000000 for 1.0 unit).
    /// The user's balance must cover this amount plus the execution fee.
    pub amount: u128,

    /// The minimum amount of liquidity shares expected in return.
    /// This provides slippage protection - if the actual shares received would be
    /// less than this amount, the transaction will fail.
    /// Should be calculated based on current market conditions and acceptable slippage.
    /// Also uses 20 decimal places precision.
    pub min_amount_out: u128,
}

impl MarketDetails {
    pub fn add_liquidity_to_market(
        &mut self,
        params: AddLiquidityToMarketParams,
    ) -> LiquidityOperationResult {
        let active_price = self.pricing_manager.get_price();

        self._add_liquidity_to_market_with_price(params, active_price)
    }

    pub fn _add_liquidity_to_market_with_price(
        &mut self,
        params: AddLiquidityToMarketParams,
        active_price: Option<u128>,
    ) -> LiquidityOperationResult {
        let AddLiquidityToMarketParams {
            amount,
            min_amount_out,
        } = params;

        let Self {
            mut liquidity_state,
            ..
        } = *self;

        let HouseLiquidityState {
            mut total_deposit,
            mut total_liquidity_shares,
            mut free_liquidity,
            mut current_house_bad_debt,
            current_longs_reserve,
            current_shorts_reserve,
            ..
        } = liquidity_state;

        let liquidity_shares_out = if total_liquidity_shares == 0
            && // if there is no positions in the market
             current_shorts_reserve == 0 && current_longs_reserve ==0
        {
            amount
        } else {
            let Some(price) = active_price else {
                return LiquidityOperationResult::Waiting { id: None };
            };

            let house_value = self._house_value(price);

            if house_value == 0 {
                //if total_liquidity_shares is not zero but house value is zero
                // undefined behaviour can occur and so the operation should fail
                return LiquidityOperationResult::Failed("House value is zero".to_string());
            } else {
                // cap during high pnl

                mul_div(amount, total_liquidity_shares, house_value)
            }
        };

        if liquidity_shares_out < min_amount_out {
            return LiquidityOperationResult::Failed(
                "Liquidity shares out is less than min amount out".to_string(),
            );
        }
        // increase total deposit
        total_deposit += amount;

        // attempts to cancel out  bad debt before increasing free_liquidity
        let repaid_bad_debt = (current_house_bad_debt).min(amount);
        if repaid_bad_debt == current_house_bad_debt {
            // amount is enough to cancel off  bad debt ;

            free_liquidity += amount - repaid_bad_debt;
        }
        current_house_bad_debt -= repaid_bad_debt;

        total_liquidity_shares += liquidity_shares_out;

        liquidity_state = HouseLiquidityState {
            total_deposit,
            total_liquidity_shares,
            free_liquidity,
            current_house_bad_debt,
            ..liquidity_state
        };

        self.liquidity_state = liquidity_state;

        return LiquidityOperationResult::Settled {
            amount_out: liquidity_shares_out,
        };
    }
}

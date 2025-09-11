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
        let price_update = self.pricing_manager.get_price();
        //.get_price_within_interval(MAX_ALLOWED_PRICE_CHANGE_INTERVAL);
        self._add_liquidity_to_market_with_price(params, price_update)
    }

    pub fn _add_liquidity_to_market_with_price(
        &mut self,
        params: AddLiquidityToMarketParams,
        price: u128,
    ) -> LiquidityOperationResult {
        let AddLiquidityToMarketParams {
            amount,
            min_amount_out,
        } = params;

        // cap during high pnl
        let house_value = self._house_value(price);

        let Self {
            liquidity_state: liquidity_manager,
            ..
        } = self;

        let HouseLiquidityState {
            mut total_deposit,
            mut total_liquidity_tokens_minted,
            mut free_liquidity,
            current_house_bad_debt: mut bad_debt,
            ..
        } = *liquidity_manager;

        let liquidity_tokens_to_mint = if house_value == 0 {
            amount
        } else {
            mul_div(amount, total_liquidity_tokens_minted, house_value)
        };

        if liquidity_tokens_to_mint < min_amount_out {
            return LiquidityOperationResult::Failed;
        }
        // increase total deposit
        total_deposit += amount;

        // attempts to cancel out  bad debt before increasing free_liquidity
        let repaid_bad_debt = (bad_debt).min(amount);
        if repaid_bad_debt == bad_debt {
            // amount is enough to cancel off  bad debt ;

            free_liquidity += amount - repaid_bad_debt;
        }
        bad_debt -= repaid_bad_debt;

        total_liquidity_tokens_minted += liquidity_tokens_to_mint;

        *liquidity_manager = HouseLiquidityState {
            total_deposit,
            total_liquidity_tokens_minted,
            free_liquidity,
            current_house_bad_debt: bad_debt,
            ..*liquidity_manager
        };

        return LiquidityOperationResult::Settled {
            amount_out: liquidity_tokens_to_mint,
        };
    }
}

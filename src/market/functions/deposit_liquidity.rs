use crate::{
    add_liquidity::add_liquidity_params::AddLiquidityToMarketParams,
    market::market_details::{LiquidityOperationResult, MarketDetails},
    math::math::mul_div,
};

use crate::market::components::liquidity_manager::HouseLiquidityManager;

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

        let house_value = self._house_value(price);

        let Self {
            liquidity_manager, ..
        } = self;

        let HouseLiquidityManager {
            total_deposit,
            total_liquidity_tokens_minted,
            free_liquidity,
            bad_debt,
            ..
        } = liquidity_manager;

        let liquidity_tokens_to_mint = if house_value == 0 {
            amount
        } else {
            mul_div(amount, *total_liquidity_tokens_minted, house_value)
        };

        if liquidity_tokens_to_mint < min_amount_out {
            return LiquidityOperationResult::Failed;
        }
        *total_deposit += amount;

        let repaid_bad_debt = (*bad_debt).min(amount);
        if repaid_bad_debt == *bad_debt {
            // amount is enough to repay bad debt ;

            *free_liquidity += amount - repaid_bad_debt;
        }
        *bad_debt -= repaid_bad_debt;

        *total_liquidity_tokens_minted += liquidity_tokens_to_mint;

        return LiquidityOperationResult::Settled {
            amount_out: liquidity_tokens_to_mint,
        };
    }
}

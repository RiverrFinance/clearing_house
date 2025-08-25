use crate::market::components::liquidity_manager::HouseLiquidityManager;
use crate::market::market_details::{LiquidityOperationResult, MarketDetails};
use crate::math::math::mul_div;
use crate::remove_liquidity::remove_liquidity_params::RemoveLiquidityFromMarketParams;

impl MarketDetails {
    pub fn remove_liquidity_from_market(
        &mut self,
        params: RemoveLiquidityFromMarketParams,
    ) -> LiquidityOperationResult {
        let price_update = self.pricing_manager.get_price();

        self._remove_liquidity_from_market_with_price(params, price_update)
    }

    pub fn _remove_liquidity_from_market_with_price(
        &mut self,
        params: RemoveLiquidityFromMarketParams,
        price: u128,
    ) -> LiquidityOperationResult {
        let RemoveLiquidityFromMarketParams {
            amount_in,
            min_amount_out,
        } = params;
        //
        let house_value = self._house_value(price);

        let Self {
            liquidity_manager, ..
        } = self;

        let HouseLiquidityManager {
            total_deposit,
            total_liquidity_tokens_minted,
            free_liquidity,
            ..
        } = liquidity_manager;

        if *total_liquidity_tokens_minted == 0 {
            return LiquidityOperationResult::Failed;
        }

        let amount_of_assets_out = mul_div(house_value, amount_in, *total_liquidity_tokens_minted);

        let amount_available = amount_of_assets_out.min(*free_liquidity);

        if amount_available < min_amount_out {
            return LiquidityOperationResult::Failed;
        }

        *free_liquidity -= amount_available;
        *total_deposit -= amount_available;

        LiquidityOperationResult::Settled {
            amount_out: amount_available,
        }
    }
}

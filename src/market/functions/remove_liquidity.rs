use crate::market::components::liquidity_state::HouseLiquidityState;
use crate::market::market_details::{LiquidityOperationResult, MarketDetails};
use crate::math::math::mul_div;

pub struct RemoveLiquidityFromMarketParams {
    /// The amount of liquidity shares to remove from the market.
    /// This should be specified with 20 decimal places precision (e.g., 10000000000000000000000 for 1.0 shares).
    /// The user's balance must cover this amount.
    pub amount_in: u128,

    /// The minimum amount of quote asset expected in return.
    /// This provides slippage protection - if the actual shares received would be
    /// less than this amount, the transaction will fail.
    /// Should be calculated based on current market conditions and acceptable slippage.
    /// Also uses 20 decimal places precision for quote asset.
    pub min_amount_out: u128,
}

impl MarketDetails {
    pub fn remove_liquidity_from_market(
        &mut self,
        params: RemoveLiquidityFromMarketParams,
    ) -> LiquidityOperationResult {
        let active_price = self.pricing_manager.get_price();

        self._remove_liquidity_from_market_with_price(params, active_price)
    }

    pub fn _remove_liquidity_from_market_with_price(
        &mut self,
        params: RemoveLiquidityFromMarketParams,
        active_price: Option<u128>,
    ) -> LiquidityOperationResult {
        let RemoveLiquidityFromMarketParams {
            amount_in,
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
            ..
        } = liquidity_state;

        if total_liquidity_shares == 0 {
            return LiquidityOperationResult::Failed("Total liquidity shares is zero".to_string());
        }

        let Some(price) = active_price else {
            return LiquidityOperationResult::Waiting { id: None };
        };
        //
        let house_value = self._house_value(price);

        let amount_of_assets_out = mul_div(house_value, amount_in, total_liquidity_shares);

        let amount_available = amount_of_assets_out.min(free_liquidity);

        if amount_available < min_amount_out {
            return LiquidityOperationResult::Failed(
                "Amount available is less than min amount out".to_string(),
            );
        }

        free_liquidity -= amount_available;
        total_deposit -= amount_available;
        total_liquidity_shares -= amount_in;

        liquidity_state = HouseLiquidityState {
            total_deposit,
            total_liquidity_shares,
            free_liquidity,
            ..liquidity_state
        };

        self.liquidity_state = liquidity_state;

        LiquidityOperationResult::Settled {
            amount_out: amount_available,
        }
    }
}

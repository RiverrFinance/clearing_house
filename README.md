# Riverr FInance Clearing House

smart contract for Clearing House Actor.

# General Overview

This section provides a general overview of how the system works.

For a Technical Overview, please see the section further below.

## Clearing House

The house comprises of markets which share the same margin/collateral token called the House Token

## Markets

Each Markets support perp trading, they are created by specifying an index/base token token .

Examples:

- ETH/USD market with collateral token as USD and index token as ETH
- BTC/USD market with collateral token as USD, and index token as BTC
- USD/ICP market with collateral token as ICP, and index token as SOL

Liquidity providers deposit house token to mint liquidity tokens.

The house token is used to back both long positions and short positions.

Liquidity providers take on the profits and losses of traders for the market that they provide liquidity for.

Having separate markets allows for risk isolation, liquidity providers are only exposed to the markets that they deposit into, this potentially allow for permissionless listings.

Traders can use margin/house token as collateral for opening a position in all available markets .

## Features

The contracts support the following main features:

- Deposit and withdrawal of liquidity
- Leverage Trading (Perps, Long / Short)
- Market orders for now , with limit orders, stop-loss, take-profit orders to be implemented later

## Oracle System

To avoid expensive and long wait time for oracle price, most actions require two steps to execute:

- User sends transaction to a particular market with request details, e.g. deposit / withdraw liquidity, swap, increase / decrease position
- checks if the curent price cached in that market was updated is within a certain interval and if it is the action is executed immediately
- If the time interval is greater than the set threshold ,the transaction is stored as a price waiting transaction and a price update transaction is triggered through a timer
- After the price is fetched and updated all pending price waiting operation is executed

Prices are provided by the DFINITY Exchange Rate Canister

Prices stored within market represent the price of one unit of the index token with respect to the house token using a value with 20 decimals of precision.

Representing the prices in this way allows for conversions between token amounts and fiat values to be simplified, e.g. to calculate the fiat value of a given number of tokens the calculation would just be: token amount \* oracle price, to calculate the token amount for a fiat value it would be: fiat value / oracle price.

## Fees and Pricing

Funding fees keep longs / shorts balanced while reducing the risk of price manipulation.

- Funding fees: if there is an imbalance of longs / shorts, the larger side pays a funding fee to the smaller side
- Borrowing fees: to avoid a user opening equal longs / shorts and unnecessarily taking up capacity

## Timers

Each market has its own timer which handles both collection of borrowing fees hourly and settling funding fees in intervals :

## Structure

There are a few main types of stuctures :

- Clearing House Canister for each margin asset called **House Asset/Token** which handles both deposit ,trading and liquidity provision for that market
- Markets within Clearing House Canister comprises of all markets with the House asset as quote asset i.e BTC/USD,ETH/USD,ICP/USD with USD as house/ quote asset
- Liquidity Manager for each Market which records all liquidity provision paramters ,it stores amount of assets deposited into a market by liquidity providers ,details about the liquidity provision token minted out as liquidity share for providing liquidity

## RLV

Short for Riverr Liquidity Vault: a wrapper of multiple markets . Liquidity is automatically rebalanced between underlying markets based on markets utilisation.

# Technical Overview

This section provides a technical description of the contracts.

## Markets

Markets are created using `createMarket`, this also creates a a Market Liquidity Token .

The Market Liquidity Token is used to keep track of liquidity providers share of the market pool and to store the tokens for each market.

At any point in time, the price of a Market Liquidity Token is `(worth of market pool) / Market Liquidity Token totalSupply`, the function `Market.house_value()` can be used to retrieve this value.

The worth of the market pool is the sum of

- worth of all tokens deposited into the pool
- total pending PnL of all open positions
- total pending borrow fees of all open positions

## Liquidity Provider Deposits

Deposits adds House asset market's pool and mints Market Liquidity Tokens to the depositor.

Requests for deposits are created by calling depositLiquidity function , specifying:

- the market to deposit into
- amount of house asset(tokens) to deposit
- minimum amount for Market Liquidity Token that should be received

Deposit requests are executed using the Oracle system described above

The amount of MarketTokens to be minted, before fees and price impact, is calculated as `(amount of  tokens deposited * Market Liquidity Token total supply) / (worth of market pool) `.

## Liquidity Provider Withdrawals

Withdrawals burn Market Liquidity Tokens for house asset in a market's pool.

Requests for withdrawals are created by calling withdrawLiquidity function, specifying:

- the market to withdraw from
- amount of Market Liquidity Token (asset) to deposit
- minimum amount for House asset that should be received

Deposit requests are executed using the Oracle system described above

The amount of long or short tokens to be redeemed, before fees and price impact, is calculated as `(amount  of liquidity token being burnt * worth of market pool ) / (Market Liquidity Token total suppky )`.

# Funding Fees

Funding fees incentivise the balancing of long and short positions, the side with the larger open interest pays a funding fee to the side with the smaller open interest.

Funding fees for the larger side is calculated as `(funding factor per second) * (open interest imbalance) ^ (funding exponent factor) / (total open interest)`.

For example if the funding factor per second is 1 / 50,000, and the funding exponent factor is 1, and the long open interest is $150,000 and the short open interest is $50,000 then the funding fee per second for longs would be `(1 / 50,000) * 100,000 / 200,000 => 0.00001 => 0.001%`.

The funding fee per second for shorts would be `-0.00001 * 150,000 / 50,000 => 0.00003 => -0.003%`.

It is also possible to set a fundingIncreaseFactorPerSecond value, this would result in the following funding logic:

- The `longShortImbalance` is calculated as `[abs(longOpenInterest - shortOpenInterest)]^ fundingExponentFactor / totalOpenInterest] `
- If the current `longShortImbalance` is more than the `thresholdForStableFunding`, then the funding rate will increase by `longShortImbalance * fundingIncreaseFactorPerSecond`
- If the current `longShortImbalance` is more than `thresholdForDecreaseFunding` and less than `thresholdForStableFunding` and the skew is in the same direction as the funding, then the funding rate will not change
- If the current `longShortImbalance` is less than `thresholdForDecreaseFunding` and the skew is in the same direction as the funding, then the funding rate will decrease by `fundingDecreaseFactorPerSecond`

## Examples

### Example 1

- thresholdForDecreaseFunding is 3%
- thresholdForStableFunding is 5%
- fundingIncreaseFactorPerSecond is 0.0001%
- fundingDecreaseFactorPerSecond is 0.000002%
- durationInSeconds is 600 seconds
- longs are paying shorts funding
- there are more longs than shorts
- longShortImbalance is 6%

Since longShortImbalance > thresholdForStableFunding, savedFundingFactorPerSecond should increase by `0.0001% * 6% * 600 = 0.0036%`

### Example 2

- thresholdForDecreaseFunding is 3%
- thresholdForStableFunding is 5%
- fundingIncreaseFactorPerSecond is 0.0001%
- fundingDecreaseFactorPerSecond is 0.000002%
- durationInSeconds is 600 seconds
- longs are paying shorts funding
- there are more longs than shorts
- longShortImbalance is 4%

Since longs are already paying shorts, the skew is the same, and the longShortImbalance < thresholdForStableFunding, savedFundingFactorPerSecond should not change

### Example 3

- thresholdForDecreaseFunding is 3%
- thresholdForStableFunding is 5%
- fundingIncreaseFactorPerSecond is 0.0001%
- fundingDecreaseFactorPerSecond is 0.000002%
- durationInSeconds is 600 seconds
- longs are paying shorts funding
- there are more longs than shorts
- longShortImbalance is 2%

Since longShortImbalance < thresholdForDecreaseFunding, savedFundingFactorPerSecond should decrease by `0.000002% * 600 = 0.0012%`

### Example 4

- thresholdForDecreaseFunding is 3%
- thresholdForStableFunding is 5%
- fundingIncreaseFactorPerSecond is 0.0001%
- fundingDecreaseFactorPerSecond is 0.000002%
- durationInSeconds is 600 seconds
- longs are paying shorts funding
- there are more shorts than longs
- longShortImbalance is 1%

Since the skew is in the other direction, savedFundingFactorPerSecond should decrease by `0.0001% * 1% * 600 = 0.0006%`

Note that there are possible ways to game the funding fees, the funding factors should be adjusted to minimize this possibility:

- If longOpenInterest > shortOpenInterest and longShortImbalance is within thresholdForStableFunding, a user holding a short position could open a long position to increase the longShortImbalance and attempt to cause the funding fee to increase. In an active market, it should be difficult to predict when an opposing short position would be opened by someone else to earn the increased funding fee which should make this gaming difficult, the funding factors can also be adjusted to help minimize the benefit of this gaming.
- If longOpenInterest > shortOpenInterest and longShortImbalance > thresholdForStableFunding, a trader holding a long position could make multiple small trades during this time to ensure that the funding factor is continually updated instead of a larger value being used for the entirety of the duration, this should minimize the funding fee for long positions but should not decrease the funding fee below the expected rates.

# Borrowing Fees

There is a borrowing fee paid to liquidity providers, this helps prevent users from opening both long and short positions to take up pool capacity without paying any fees.

River Borrowing fees use the curve model.

To use the curve model, the keys to configure would be `BORROWING_FACTOR` and `BORROWING_EXPONENT_FACTOR`, the borrowing factor per second would be calculated as:

- borrowing_factor_per_sec = borrowing_factor \* (reserve_value^ borrowing_factor_exponent)/ pool_value_inactive
   where
- reserve value : the total amount reserved for positions
- pool_value_inactive :pool_value -= traders pnl

There is also an option to set a skipBorrowingFeeForSmallerSide flag, this would result in the borrowing fee for the smaller side being set to zero. For example, if there are more longs than shorts and skipBorrowingFeeForSmallerSide is true, then the borrowing fee for shorts would be zero.

# Fees

There are configurable fees per market

Execution fees are also estimated and accounted for on creation of deposit, withdrawal, order requests so that keepers can execute transactions at a close to net zero cost.

# Reserve Amounts

A market should be able to fully pay positions profits,riverr enables ensures by making a reserve for everyu position opend ,the reserve for every position serves as the maximum positive pnl for that position and traders pick this figure when opening a position ,this also serves as a measure of the maximum loss by the house for a position opened by a trader .

In order to prevent traders from setting excessively high pnl ,a MAX_PNL_FACTOR is configured for every market hence only a certain percentage of open interest can be set as pnl

- exmaple: MAX_PNL_FACTOR of 0.9 (90%) means a the maximum set reserve amount for when opening a position of open interest $1,000,000 is 0.9 \* 1,000,000 = $900,000

Markets have a MAX_RSERVE_FACTOR factor that allows position reserve to be capped to a percentage of the pool value, this reduces the impact of profits of positions on liquidity providers.

# Market Token Price

The price of a market token depends on the worth of the assets in the pool, and the net pending PnL of traders' open positions.

It is possible for the pending PnL to be capped, the factors used to calculate the market token price can differ depending on the activity:

- Keys.MAX_PNL_FACTOR_FOR_DEPOSITS: this is the PnL factor cap when calculating the market token price for deposits
- Keys.MAX_PNL_FACTOR_FOR_WITHDRAWALS: this is the PnL factor cap when calculating the market token price for withdrawals
- Keys.MAX_PNL_FACTOR_FOR_TRADERS: this is the PnL factor cap when calculating the market token price for closing a position

These different factors can be configured to help liquidity providers manage risk and to incentivise deposits when needed, e.g. capping of trader PnL helps cap the amount the market token price can be decreased by due to trader PnL, capping of PnL for deposits and withdrawals can lead to a lower market token price for deposits compared to withdrawals which can incentivise deposits when pending PnL is high.

# Parameters

- fundingFactor: This is the "funding factor per second" value described in the "Funding Fees" section
- borrowingFactorForLongs: This is the "borrowing factor" for long positions described in the "Borrowing Fees" section
- borrowingFactorForShorts: This is the "borrowing factor" for short positions described in the "Borrowing Fees" section
- borrowingExponentFactorForLongs: This is the "borrowing exponent factor" for long positions described in the "Borrowing Fees" section
- borrowingExponentFactorForShorts: This is the "borrowing exponent factor" for long positions described in the "Borrowing Fees" section

# Known Issues

## Tokens

- Rebasing tokens, tokens that change balance on transfer, with token burns, tokens with callbacks are not compatible with the system and should not be whitelisted

## Market Liquidity Token Price

- It is rare but possible for a pool's value to become negative, this can happen since the impactPoolAmount and pending PnL is subtracted from the worth of the tokens in the pool
- Due to the difference in positive and negative position price impact, there can be a build up of virtual token amounts in the position impact pool which would affect the pricing of market tokens, the position impact pool should be gradually distributed if needed

# Commands

To compile contracts:

```sh
cargo build --release --target wasm32-unknown-unknown --package clearing_house

```

To extract .did file

```sh
   candid-extractor target/wasm32-unknown-unknown/release/clearing_house.wasm > src/clearing_house.did
```

To run all tests:

```sh
cargo test

```

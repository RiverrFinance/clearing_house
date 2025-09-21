import type { Agent } from "@icp-sdk/core/agent";
import { createActor as createClearingHouseActor } from "../../src/declarations/clearing_house/index.js";
import type {
  AddLiquidityParams,
  CreateMarketParams,
  DepositParams,
  LiquidityOperationResult,
  MarketDetails,
  MarketState,
  QueryMarketDetailsResult,
  _SERVICE,
} from "../../src/declarations/clearing_house/clearing_house.did.js";
import type { Principal } from "@icp-sdk/core/principal";

export class ClearingHouseActor {
  clearingHouse: _SERVICE;
  constructor(
    private readonly agent: Agent,
    private readonly canisterId: Principal
  ) {
    this.clearingHouse = createClearingHouseActor(canisterId, {
      agent,
    });
  }
  public async createMarket(params: CreateMarketParams): Promise<bigint> {
    let result = await this.clearingHouse.createNewMarket(params);

    return result;
  }

  public async queryMarketDetails(
    marketIndex: bigint
  ): Promise<QueryMarketDetailsResult> {
    let result = await this.clearingHouse.queryMarketDetails(marketIndex);

    return result;
  }

  public async getMarketDetails(marketIndex: bigint): Promise<MarketDetails> {
    let result = await this.clearingHouse.get_market_details(marketIndex);

    return result;
  }

  public async getUserSharesBalance(
    principal: Principal,
    marketIndex: bigint
  ): Promise<bigint> {
    let result = await this.clearingHouse.getUserMarketLiquidityShares(
      principal,
      marketIndex
    );

    return result;
  }

  public async getUserBalance(principal: Principal): Promise<bigint> {
    let result = await this.clearingHouse.getUserBalance(principal);

    return result;
  }

  public async addLiquidity(
    params: AddLiquidityParams
  ): Promise<LiquidityOperationResult> {
    let result = await this.clearingHouse.addLiquidity(params);

    return result;
  }

  public async depositIntoAccount(params: DepositParams): Promise<boolean> {
    let result = await this.clearingHouse.depositIntoAccount(params);

    return result;
  }
}

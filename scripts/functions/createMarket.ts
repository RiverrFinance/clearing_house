import { config } from "dotenv";

config();

import { type _SERVICE } from "../../src/declarations/clearing_house/clearing_house.did.js";
import { HttpAgent, type ActorSubclass } from "@dfinity/agent";
import { Ed25519KeyIdentity } from "@icp-sdk/core/identity";

import { ClearingHouseActor } from "../actors/ClearingHouseActor.js";
import { createIdentityFromPrivateKey } from "../utils/createIdentityFromPrivateKey.js";

import { toParseUnits, toPrecision } from "../utils/conversion.js";
import { Principal } from "@icp-sdk/core/principal";

export const createMarket = async () => {
  if (process.env.PRIVATE_KEY_HEX) {
    const identity: Ed25519KeyIdentity = createIdentityFromPrivateKey(
      process.env.PRIVATE_KEY_HEX
    );
    console.log(identity.getPrincipal().toString());
    // const agent = await HttpAgent.create({
    //   host: "https://icp-api.io",
    //   identity,
    // });
    // const clearingHouseActor = new ClearingHouseActor(
    //   agent,
    //   Principal.fromText("5ch5r-aiaaa-aaaao-a4pma-cai")
    // );
    // let params: CreateMarketParams = {
    //   assetPricingDetails: {
    //     class: { Cryptocurrency: null },
    //     symbol: "BTC",
    //   },
    //   initState: {
    //     liquidationFactor: toPrecision(0.01), // 1 percent
    //     maxLeverageFactor: toPrecision(10), // 10x
    //     maxReserveFactor: toPrecision(0.5), // 50% price change
    //   },
    //   shortsBaseBorrowingFactor: toPrecision(5 * 10 ** -9),
    //   longsBaseBorrowingFactor: toPrecision(5 * 10 ** -9),
    //   longsBorrowingExponentFactor: toPrecision(1),
    //   shortsBorrowingExponentFactor: toPrecision(1),
    //   fundingFactor: toPrecision(2 * 10 ** -8),
    //   fundingExponentFactor: toPrecision(1),
    //   shortsMaxReserveFactor: toPrecision(0.3), // 30%
    //   longsMaxReserveFactor: toPrecision(0.3), //30%
    // };
    // console.log(params.shortsBaseBorrowingFactor);
    // let marketIndex = await clearingHouseActor.createMarket(params);
    // console.log(marketIndex);
    // try {
    //   let marketDetails = await clearingHouseActor.getMarketDetails(0n);
    //   console.log(marketDetails);
    // } catch (error) {
    //   console.log(error);
    // }
  }
  // console.log(params.funding_factor);
};

createMarket().catch((error) => {
  console.log(error);
  process.exit(1);
});

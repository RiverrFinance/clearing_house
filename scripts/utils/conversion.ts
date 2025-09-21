import { parseUnits } from "@ethersproject/units";

const PRECISION_DECIMALS = 20;
export const toParseUnits = (value: number): bigint => {
  return parseUnits(value.toString(), PRECISION_DECIMALS).toBigInt();
};

export const toPrecision = (value: number): bigint => {
  return BigInt(value * 10 ** PRECISION_DECIMALS);
};

import { ApiPromise } from "@polkadot/api";

export function log(...msg: any[]) {
  if (process.argv && process.argv[2] && process.argv[2] === "--printlogs") {
    console.log(...msg);
  }
}

export const printTokens = (api: ApiPromise, tokens: bigint, decimals = 2, pad = 9) => {
  return `${(
    Math.ceil(Number(tokens / 10n ** BigInt(api.registry.chainDecimals[0] - decimals))) /
    10 ** decimals
  )
    .toString()
    .padStart(pad)} ${api.registry.chainTokens[0]}`;
};

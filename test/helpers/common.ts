import { ApiPromise } from "@polkadot/api";
import { u32 } from "@polkadot/types";
import Bottleneck from "bottleneck";

export function rateLimiter() {
  const settings =
    process.env.SKIP_RATE_LIMITER === "true" ? {} : { maxConcurrent: 10, minTime: 150 };
  return new Bottleneck(settings);
}

export async function checkTimeSliceForUpgrades(
  api: ApiPromise,
  blockNumbers: number[],
  currentVersion: u32
) {
  const apiAt = await api.at(await api.rpc.chain.getBlockHash(blockNumbers[0]));
  const onChainRt = (await apiAt.query.system.lastRuntimeUpgrade()).unwrap().specVersion;
  return { result: !onChainRt.eq(currentVersion), specVersion: onChainRt };
}

// Sort dict by key
export function sortObjectByKeys(o) {
  return Object.keys(o)
    .sort()
    .reduce((r, k) => ((r[k] = o[k]), r), {});
}
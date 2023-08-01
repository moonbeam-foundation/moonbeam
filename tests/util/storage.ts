import { ApiPromise } from "@polkadot/api";
import { rateLimiter } from "../util/common";

const debug = require("debug")("smoke:storage-query");

// Timer must be wrapped to be passed
const startReport = (total: () => number) => {
  let t0 = performance.now();
  let timer: NodeJS.Timeout = undefined;

  const report = () => {
    const t1 = performance.now();
    const duration = t1 - t0;
    const qps = total() / (duration / 1000);
    const used = process.memoryUsage().heapUsed / 1024 / 1024;
    debug(`Queried ${total()} keys @ ${qps.toFixed(0)} keys/sec, ${used.toFixed(0)} MB heap used`);

    timer = setTimeout(report, 5000);
  };
  timer = setTimeout(report, 5000);

  const stopReport = () => {
    clearTimeout(timer);
  };

  return stopReport;
};

export function splitPrefix(prefix: string) {
  return new Array(256).fill(0).map((_, i) => `${prefix}${i.toString(16).padStart(2, "0")}`);
}

// Only works with keys longer than keyPrefix
// Is effective only on well spread keys
export async function concurrentGetKeys(api: ApiPromise, keyPrefix: string, blockHash: string) {
  const maxKeys = 1000;
  let total = 0;

  let prefixes = splitPrefix(keyPrefix);
  const limiter = rateLimiter();
  const stopReport = startReport(() => total);

  try {
    const allKeys = await Promise.all(
      prefixes.map(async (prefix) =>
        limiter.schedule(async () => {
          let keys = [];
          let startKey = null;
          while (true) {
            const result = await (api as any)._rpcCore.provider.send("state_getKeysPaged", [
              prefix,
              maxKeys,
              startKey,
              blockHash,
            ]);
            total += result.length;
            keys.push(...result);
            if (result.length != maxKeys) {
              break;
            }
            startKey = result[result.length - 1];
          }
          global.gc();
          return keys;
        })
      )
    );
    return allKeys.flat().sort();
  } finally {
    stopReport();
  }
}

export async function queryUnorderedRawStorage(
  api: ApiPromise,
  keys: string[],
  blockHash: string
): Promise<
  {
    key: `0x${string}`;
    value: string;
  }[]
> {
  // @ts-expect-error _rpcCore is not yet exposed
  const result = await api._rpcCore.provider.send("state_queryStorageAt", [keys, blockHash]);

  return result[0].changes.map((pair) => ({
    value: pair[1],
    key: pair[0],
  }));
}

export async function processAllStorage(
  api: ApiPromise,
  storagePrefix: string,
  blockHash: string,
  processor: (batchResult: { key: `0x${string}`; value: string }[]) => void
) {
  const maxKeys = 1000;
  let total = 0;
  let prefixes = splitPrefix(storagePrefix);
  const limiter = rateLimiter();
  const stopReport = startReport(() => total);

  try {
    await Promise.all(
      prefixes.map(async (prefix) =>
        limiter.schedule(async () => {
          let startKey = null;
          while (true) {
            // @ts-expect-error _rpcCore is not yet exposed
            const keys = await api._rpcCore.provider.send("state_getKeysPaged", [
              prefix,
              maxKeys,
              startKey,
              blockHash,
            ]);
            // @ts-expect-error _rpcCore is not yet exposed
            const response = await api._rpcCore.provider.send("state_queryStorageAt", [
              keys,
              blockHash,
            ]);

            if (!response[0]) {
              console.log(response);
            }

            processor(response[0].changes.map((pair) => ({ key: pair[0], value: pair[1] })));
            total += keys.length;

            if (keys.length != maxKeys) {
              break;
            }
            startKey = keys[keys.length - 1];
          }
        })
      )
    );
  } finally {
    stopReport();
  }
  await limiter.disconnect();
}

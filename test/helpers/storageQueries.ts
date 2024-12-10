import type { ApiPromise } from "@polkadot/api";
import Debugger from "debug";
import { rateLimiter } from "./common.js";

const log = Debugger("test:storageQuery");

const startReport = (total: () => number) => {
  const t0 = performance.now();
  let timer: NodeJS.Timeout;

  const report = () => {
    const t1 = performance.now();
    const duration = t1 - t0;
    const qps = total() / (duration / 1000);
    const used = process.memoryUsage().heapUsed / 1024 / 1024;
    log(
      `🔍️ Queried ${total()} keys @ ${qps.toFixed(0)} keys/sec,` +
        ` ${used.toFixed(0)} MB heap used\n`
    );

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

export async function processAllStorage(
  api: ApiPromise,
  storagePrefix: string,
  blockHash: string,
  processor: (batchResult: { key: `0x${string}`; value: string }[]) => void
) {
  const maxKeys = 1000;
  let total = 0;
  const prefixes = splitPrefix(storagePrefix);
  const limiter = rateLimiter();
  const stopReport = startReport(() => total);

  try {
    await Promise.all(
      prefixes.map(async (prefix) =>
        limiter.schedule(async () => {
          let startKey: string | undefined = undefined;
          for (;;) {
            // @ts-expect-error _rpcCore is not yet exposed
            const keys: string = await api._rpcCore.provider.send("state_getKeysPaged", [
              prefix,
              maxKeys,
              startKey,
              blockHash,
            ]);

            if (!keys.length) {
              break;
            }

            // @ts-expect-error _rpcCore is not yet exposed
            const response = await api._rpcCore.provider.send("state_queryStorageAt", [
              keys,
              blockHash,
            ]);

            try {
              processor(
                response[0].changes.map((pair: [string, string]) => ({
                  key: pair[0],
                  value: pair[1],
                }))
              );
            } catch (e) {
              console.log(`Error processing ${prefix}: ${e}`);
            }

            total += keys.length;

            if (keys.length !== maxKeys) {
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

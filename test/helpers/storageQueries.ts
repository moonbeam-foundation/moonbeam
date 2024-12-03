import { ApiPromise } from "@polkadot/api";
import Debugger from "debug";
import { rateLimiter } from "./common.js";
import { randomAsHex } from "@polkadot/util-crypto";

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
          loop: for (;;) {
            // @ts-expect-error _rpcCore is not yet exposed
            const keys: string = await api._rpcCore.provider.send("state_getKeysPaged", [
              prefix,
              maxKeys,
              startKey,
              blockHash,
            ]);

            if (!keys.length) {
              break loop;
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
              break loop;
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

export async function processRandomStoragePrefixes(
  api: ApiPromise,
  storagePrefix: string,
  blockHash: string,
  processor: (batchResult: { key: `0x${string}`; value: string }[]) => void,
  override: string = ""
) {
  const maxKeys = 1000;
  let total = 0;
  const preFilteredPrefixes = splitPrefix(storagePrefix);
  const chanceToSample = 0.05;
  const prefixes = override
    ? [override]
    : preFilteredPrefixes.filter(() => Math.random() < chanceToSample);
  console.log(`Processing ${prefixes.length} prefixes: ${prefixes.join(", ")}`);
  const limiter = rateLimiter();
  const stopReport = startReport(() => total);

  try {
    await Promise.all(
      prefixes.map(async (prefix) =>
        limiter.schedule(async () => {
          let startKey: string | undefined = undefined;
          loop: for (;;) {
            // @ts-expect-error _rpcCore is not yet exposed
            const keys: string = await api._rpcCore.provider.send("state_getKeysPaged", [
              prefix,
              maxKeys,
              startKey,
              blockHash,
            ]);

            if (!keys.length) {
              break loop;
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
              console.log(`Replace the empty string in smoke/test-ethereum-contract-code.ts:L51
                with the prefix to reproduce`);
            }

            total += keys.length;

            if (keys.length !== maxKeys) {
              break loop;
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

export async function getStartingKeySample(api: ApiPromise, prefix: string, blockHash: string) {
  // @ts-expect-error _rpcCore is not yet exposed
  const res: string = await api._rpcCore.provider.send("state_getKeysPaged", [
    prefix,
    1,
    "",
    blockHash,
  ]);

  return res[0];
}

export const extractStorageKeyComponents = (storageKey: string) => {
  // The full storage key is composed of
  // - The 0x prefix (2 characters)
  // - The module prefix (32 characters)
  // - The method name (32 characters)
  // - The parameters (variable length)
  const regex = /(?<moduleKey>0x[a-f0-9]{32})(?<fnKey>[a-f0-9]{32})(?<paramsKey>[a-f0-9]*)/i;
  const match = regex.exec(storageKey);

  if (!match) {
    throw new Error("Invalid storage key format");
  }

  const { moduleKey, fnKey, paramsKey } = match.groups!;
  return {
    moduleKey,
    fnKey,
    paramsKey,
  };
};

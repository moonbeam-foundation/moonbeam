import "@moonbeam-network/api-augment";

import fs from "node:fs";
import { ApiPromise, WsProvider } from "@polkadot/api";
import yargs from "yargs";
import { hideBin } from "yargs/helpers";

const DEFAULT_ENDPOINTS = {
  moonbase: "wss://trace.api.moonbase.moonbeam.network",
  moonbeam: "wss://trace.api.moonbeam.network",
  moonriver: "wss://wss.moonriver.moonbeam.network",
} as const;

type Chain = keyof typeof DEFAULT_ENDPOINTS;

const argv = yargs(hideBin(process.argv))
  .usage("Usage: $0 --chain <moonbase|moonbeam|moonriver>")
  .options({
    chain: {
      choices: Object.keys(DEFAULT_ENDPOINTS) as Chain[],
      demandOption: true,
      description: "Moonbeam network to inspect",
    },
    endpoint: {
      type: "string",
      description: "Override websocket endpoint",
    },
    "max-depth": {
      type: "number",
      default: 250,
      description: "Finalized blocks to scan backwards",
    },
    "min-descendants": {
      type: "number",
      default: 2,
      description: "Minimum relay parent descendants required",
    },
    "github-env": {
      type: "string",
      description: "Append CHOPSTICKS_BLOCK to this GitHub Actions env file",
    },
  })
  .parseSync();

const getRelayParentDescendantsLength = (extrinsic: any): number | undefined => {
  if (
    extrinsic.method?.section !== "parachainSystem" ||
    extrinsic.method?.method !== "setValidationData"
  ) {
    return undefined;
  }

  const inherentData = extrinsic.method.args[0] as any;
  const descendants = inherentData?.relayParentDescendants;

  return typeof descendants?.length === "number" ? descendants.length : undefined;
};

const wait = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

const retryRpc = async <T>(
  operation: () => Promise<T>,
  options: { maxAttempts?: number; backoffMs?: number } = {}
): Promise<T> => {
  const maxAttempts = options.maxAttempts ?? 4;
  const backoffMs = options.backoffMs ?? 500;

  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    try {
      return await operation();
    } catch (error) {
      if (attempt === maxAttempts) {
        throw error;
      }

      await wait(backoffMs * 2 ** (attempt - 1));
    }
  }

  throw new Error("RPC retry attempts exhausted");
};

const main = async () => {
  const endpoint = argv.endpoint ?? DEFAULT_ENDPOINTS[argv.chain];
  const provider = new WsProvider(endpoint);
  const api = await ApiPromise.create({ provider, noInitWarn: true });

  try {
    const finalizedHash = await retryRpc(() => api.rpc.chain.getFinalizedHead());
    const finalizedHeader = await retryRpc(() => api.rpc.chain.getHeader(finalizedHash));
    const finalizedNumber = finalizedHeader.number.toNumber();

    const oldestNumber = Math.max(0, finalizedNumber - argv.maxDepth);

    for (let number = finalizedNumber; number >= oldestNumber; number--) {
      const hash = await retryRpc(() => api.rpc.chain.getBlockHash(number));
      const block = await retryRpc(() => api.rpc.chain.getBlock(hash));
      const descendantsLength = block.block.extrinsics
        .map(getRelayParentDescendantsLength)
        .find((length) => length !== undefined);

      if (descendantsLength === undefined) {
        continue;
      }

      if (descendantsLength >= argv.minDescendants) {
        console.log(
          `Selected ${argv.chain} block #${number} with ${descendantsLength} relay parent descendants`
        );
        console.log(`CHOPSTICKS_BLOCK=${number}`);

        if (argv.githubEnv) {
          fs.appendFileSync(argv.githubEnv, `CHOPSTICKS_BLOCK=${number}\n`);
        }

        return;
      }
    }

    throw new Error(
      `No finalized ${argv.chain} block with at least ${argv.minDescendants} relay parent descendants found in the last ${argv.maxDepth} blocks`
    );
  } finally {
    await api.disconnect();
  }
};

await main();

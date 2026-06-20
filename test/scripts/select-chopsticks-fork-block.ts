import "@moonbeam-network/api-augment";

import fs from "node:fs";
import { ApiPromise, WsProvider } from "@polkadot/api";
import { blake2AsHex } from "@polkadot/util-crypto";
import yargs from "yargs";
import { hideBin } from "yargs/helpers";

const DEFAULT_ENDPOINTS = {
  moonbase: "wss://trace.api.moonbase.moonbeam.network",
  moonbeam: "wss://trace.api.moonbeam.network",
  moonriver: "wss://wss.moonriver.moonbeam.network",
} as const;

type Chain = keyof typeof DEFAULT_ENDPOINTS;

type SetValidationDataMetrics = {
  descendantsLength: number;
  relayChainTrieNodes: number;
  proofAnchored: boolean;
};

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
      description: "Minimum relay parent descendants required (moonbase-style inherents)",
    },
    "github-env": {
      type: "string",
      description: "Append CHOPSTICKS_BLOCK to this GitHub Actions env file",
    },
  })
  .parseSync();

const toU8a = (node: any): Uint8Array =>
  typeof node?.toU8a === "function" ? node.toU8a(true) : (node as Uint8Array);

/**
 * Verify the relay-chain state proof carried by `setValidationData` is well-formed
 * and anchored: the proof must include the root trie node, i.e. some node must
 * blake2-256-hash to `validationData.relayParentStorageRoot`.
 *
 * This is the consistency property that Chopsticks relies on when it rebuilds the
 * inherent on the forked chain; a truncated or mismatched proof is exactly what
 * triggers the runtime panic the fork-block selection is meant to avoid. It cannot
 * be satisfied by an arbitrary block that merely carries a non-empty proof.
 */
const isProofAnchored = (inherentData: any): boolean => {
  const storageRoot = inherentData?.validationData?.relayParentStorageRoot?.toHex?.();
  const trieNodes = inherentData?.relayChainState?.trieNodes;

  if (typeof storageRoot !== "string" || !trieNodes) {
    return false;
  }

  for (const node of trieNodes) {
    if (blake2AsHex(toU8a(node), 256) === storageRoot) {
      return true;
    }
  }

  return false;
};

const getSetValidationDataMetrics = (extrinsic: any): SetValidationDataMetrics | undefined => {
  if (
    extrinsic.method?.section !== "parachainSystem" ||
    extrinsic.method?.method !== "setValidationData"
  ) {
    return undefined;
  }

  const inherentData = extrinsic.method.args[0] as any;
  const descendants = inherentData?.relayParentDescendants;
  const descendantsLength =
    typeof descendants?.length === "number" ? descendants.length : undefined;

  if (descendantsLength === undefined) {
    return undefined;
  }

  const trieNodes = inherentData?.relayChainState?.trieNodes;
  const relayChainTrieNodes =
    typeof trieNodes?.size === "number"
      ? trieNodes.size
      : typeof trieNodes?.length === "number"
        ? trieNodes.length
        : 0;

  return {
    descendantsLength,
    relayChainTrieNodes,
    proofAnchored: isProofAnchored(inherentData),
  };
};

const isStableForkBlock = (metrics: SetValidationDataMetrics, minDescendants: number): boolean => {
  // The block must carry a valid, anchored relay-chain state proof regardless of
  // the inherent format.
  if (!metrics.proofAnchored) {
    return false;
  }

  // Legacy inherents (moonbase) embed the relay parent descendants; require enough
  // of them, matching the original selection behaviour.
  if (metrics.descendantsLength >= minDescendants) {
    return true;
  }

  // Production runtimes (moonbeam/moonriver) no longer populate relay parent
  // descendants and rely solely on the anchored relay-chain state proof above.
  return metrics.descendantsLength === 0;
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
      const metrics = block.block.extrinsics
        .map(getSetValidationDataMetrics)
        .find((value) => value !== undefined);

      if (metrics === undefined) {
        continue;
      }

      if (!isStableForkBlock(metrics, argv.minDescendants)) {
        continue;
      }

      console.log(
        `Selected ${argv.chain} block #${number} with ${metrics.descendantsLength} relay parent ` +
          `descendants and an anchored relay-chain state proof (${metrics.relayChainTrieNodes} trie nodes)`
      );
      console.log(`CHOPSTICKS_BLOCK=${number}`);

      if (argv.githubEnv) {
        fs.appendFileSync(argv.githubEnv, `CHOPSTICKS_BLOCK=${number}\n`);
      }

      return;
    }

    throw new Error(
      `No finalized ${argv.chain} block with an anchored relay-chain state proof ` +
        `(and at least ${argv.minDescendants} relay parent descendants for legacy inherents) ` +
        `found in the last ${argv.maxDepth} blocks`
    );
  } finally {
    await api.disconnect();
  }
};

await main();

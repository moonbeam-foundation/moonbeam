import { type DevModeContext, importJsonConfig } from "@moonwall/cli";
import type { ApiPromise } from "@polkadot/api";
import type { u32 } from "@polkadot/types";
import { EXTRINSIC_VERSION } from "@polkadot/types/extrinsic/v4/Extrinsic";
import { createMetadata, type KeyringPair, type OptionsWithMeta } from "@substrate/txwrapper-core";
import Bottleneck from "bottleneck";

export function rateLimiter(options?: Bottleneck.ConstructorOptions) {
  const settings =
    process.env.SKIP_RATE_LIMITER === "true"
      ? {}
      : { maxConcurrent: 10, minTime: 50, ...(options || {}) };

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

/**
 * Sorts the keys of an object and returns a new object with the same values,
 * but with keys in lexicographical order.
 *
 * @export
 * @param {Record<string, any>} unsortedObject - The object with unsorted keys.
 * @returns {Record<string, any>} - The new object with keys sorted.
 */
export function sortObjectByKeys(unsortedObject: Record<string, any>): Record<string, any> {
  return Object.keys(unsortedObject)
    .sort()
    .reduce((sortedObject: Record<string, any>, currentKey: string) => {
      sortedObject[currentKey] = unsortedObject[currentKey];
      return sortedObject;
    }, {});
}

export async function getMappingInfo(context: DevModeContext, authorId: string) {
  const mapping = await context.polkadotJs().query.authorMapping.mappingWithDeposit(authorId);
  if (mapping.isSome) {
    return {
      account: mapping.unwrap().account.toString(),
      deposit: mapping.unwrap().deposit.toBigInt(),
    };
  }
}

export async function getProviderPath() {
  const globalConfig = await importJsonConfig();
  const env = globalConfig.environments.find(({ name }) => name === process.env.MOON_TEST_ENV)!;
  return env.connections
    ? env.connections[0].endpoints[0].replace("ws://", "http://")
    : `http://127.0.0.1:${10000 + Number(process.env.VITEST_POOL_ID || 1) * 100}`;
}

export async function localViemNetworkDetails(api: ApiPromise) {
  const id = (await api.rpc.eth.chainId()).toNumber();
  const name = (await api.rpc.system.chain()).toString();
  const network = api.consts.system.version.specName.toString();
  const symbol = (await api.rpc.system.properties()).tokenSymbol.unwrapOr("UNIT")[0].toString();
  const endpoint = await getProviderPath();
  return {
    id,
    name,
    network,
    nativeCurrency: {
      decimals: 18,
      name: symbol,
      symbol,
    },
    rpcUrls: {
      public: { http: [endpoint] },
      default: { http: [endpoint] },
    },
    // contracts: {
    //   multicall3: {
    //     address: "0xca11bde05977b3631167028862be2a173976ca11",
    //     blockCreated: 11_907_934,
    //   },
    // },
  };
}

/**
 * Signing function. Implement this on the OFFLINE signing device.
 *
 * @param pair - The signing pair.
 * @param signingPayload - Payload to sign.
 */
export function signWith(
  pair: KeyringPair,
  signingPayload: string,
  options: OptionsWithMeta
): `0x${string}` {
  const { registry, metadataRpc } = options;
  // Important! The registry needs to be updated with latest metadata, so make
  // sure to run `registry.setMetadata(metadata)` before signing.
  registry.setMetadata(createMetadata(registry, metadataRpc));

  const { signature } = registry
    .createType("ExtrinsicPayload", signingPayload, {
      version: EXTRINSIC_VERSION,
    })
    .sign(pair);

  return signature as `0x${string}`; //TODO: fix this when type problem is fixed
}

/**
 * Chunks a given array
 *
 * @param array - The array to chunk
 * @param size - Size of a chunk
 */
export function chunk<T>(array: Array<T>, size: number): Array<Array<T>> {
  const chunks = [];
  for (let i = 0; i < array.length; i += size) {
    chunks.push(array.slice(i, i + size));
  }

  return chunks;
}

/**
 * Pauses the execution for a specified duration.
 * @param durationMs The duration to sleep in milliseconds.
 * @returns A Promise that resolves after the specified duration.
 */
export async function sleep(durationMs: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, durationMs));
}

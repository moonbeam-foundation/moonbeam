// This script is expected to run against a parachain network (using launch.ts script)
import chalk from "chalk";
import yargs from "yargs";
import {
  listenBestBlocks,
  listenFinalizedBlocks,
  printRealtimeBlockDetails,
} from "./utils/monitoring";
import { ApiPromise, WsProvider } from "@polkadot/api";
import { typesBundle } from "../moonbeam-types-bundle";

import { getApiFor, isKnownNetwork, NETWORK_COLORS, NETWORK_YARGS_OPTIONS } from "./utils/networks";

const PROVIDERS = [
  "wss://wss.moonriver.moonbeam.network",
  "wss://moonriver.api.onfinality.io/public-ws",
  "ws://localhost:56992",
];

const argv = yargs(process.argv.slice(2))
  .usage("Usage: $0")
  .version("1.0.0")
  .options({
    ...NETWORK_YARGS_OPTIONS,
    finalized: {
      type: "boolean",
      default: false,
      description: "listen to finalized only",
    },
  }).argv;

const main = async () => {
  const bestHashes = {};
  let startBlock;
  let started = false;

  const nodes = await Promise.all(
    PROVIDERS.map(async (endpoint) => {
      const node = {
        endpoint,
        api: await ApiPromise.create({
          provider: new WsProvider(endpoint),
          typesBundle: typesBundle,
        }),
      };
      try {
        await node.api.isReadyOrError;
      } catch (err) {
        console.log(`✘ Couldn't set up API, is the endpoint up?. ${err.toString()}`);
        process.exit(1);
      }
      return node;
    })
  );

  listenBestBlocks(nodes[0].api, async ({ block }) => {
    if (!startBlock) {
      startBlock = block.header.number.toString();
    }
    bestHashes[block.header.number.toString()] = block.header.hash.toString();
  });

  listenFinalizedBlocks(nodes[0].api, async (blockDetails) => {
    const blockNumber = blockDetails.block.header.number;
    const blockHash = blockDetails.block.header.hash;
    if (!started && startBlock == blockNumber) {
      started = true;
    }
    const hashes = await Promise.all(
      nodes.map(async ({ endpoint, api }) => {
        try {
          return (await api.rpc.chain.getBlockHash(blockNumber.toNumber())).toString();
        } catch (e) {
          return "";
        }
      })
    );

    let isGetHashValid = true;
    for (const index in hashes) {
      if (hashes[index].toString() != blockHash.toString()) {
        console.log(
          `ERROR: ${blockDetails.block.header.number} not matching !!! (getHash ${hashes[
            index
          ].toString()} for ${nodes[index].endpoint} vs finalized ${blockHash.toString()})`
        );
        process.exit(0);
        isGetHashValid = false;
      }
    }
    const isBestHashValid = bestHashes[blockNumber.toString()] == blockHash.toString();

    printRealtimeBlockDetails(blockDetails, {
      prefix: isKnownNetwork(`moonriver`)
        ? NETWORK_COLORS[`moonriver`](`moonriver`.padStart(10, " "))
        : undefined,
      suffix: `getBlockHash: ${isGetHashValid ? chalk.green(`✓`) : chalk.red(`X`)} - ${
        !started
          ? `...waiting more best blocks`
          : `Best: ${isBestHashValid ? chalk.green(`✓`) : ``}`
      }`,
    });
    if (!isGetHashValid) {
      console.log(
        `ERROR: ${blockDetails.block.header.number} not matching !!! (getHash ${hashes.join(
          ", "
        )} vs finalized ${blockHash.toString()})`
      );
      process.exit(1);
    }
    if (!started) {
      return;
    }
    if (bestHashes[blockNumber.toString()] != blockHash.toString()) {
      console.log(
        `ERROR: ${blockDetails.block.header.number} not matching !!! (best ${
          bestHashes[blockNumber.toString()]
        } vs finalized ${blockHash.toString()})`
      );
      process.exit(1);
    }
  });
};

main();

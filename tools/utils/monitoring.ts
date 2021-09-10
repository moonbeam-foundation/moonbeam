import { ApiPromise } from "@polkadot/api";
import { Extrinsic, BlockHash, EventRecord } from "@polkadot/types/interfaces";
import { Block } from "@polkadot/types/interfaces/runtime/types";
import type { TxWithEvent } from "@polkadot/api-derive/types";
import { mapExtrinsics } from "./types";
import chalk from "chalk";
export interface BlockDetails {
  block: Block;
  blockTime: number;
  records: EventRecord[];
  txWithEvents: TxWithEvent[];
  weightPercentage: number;
}

const getBlockDetails = async (api: ApiPromise, blockHash: BlockHash) => {
  const maxBlockWeight = api.consts.system.blockWeights.maxBlock.toBigInt();
  const [{ block }, records, blockTime] = await Promise.all([
    api.rpc.chain.getBlock(blockHash),
    api.query.system.events.at(blockHash),
    api.query.timestamp.now.at(blockHash),
  ]);

  const txWithEvents = mapExtrinsics(block.extrinsics, records);
  const blockWeight = txWithEvents.reduce((totalWeight, tx, index) => {
    return totalWeight + (tx.dispatchInfo && tx.dispatchInfo.weight.toBigInt());
  }, 0n);
  return {
    block,
    blockTime: blockTime.toNumber(),
    weightPercentage: Number((blockWeight * 10000n) / maxBlockWeight) / 100,
    txWithEvents,
    records,
  } as BlockDetails;
};

interface BlockRangeOption {
  from: number;
  to: number;
  concurrency?: number;
}
// Explore all blocks for the given range
// fromBlockNumber and toBlockNumber included
export const exploreBlockRange = async (
  api: ApiPromise,
  { from, to, concurrency = 1 }: BlockRangeOption,
  callBack: (blockDetails: BlockDetails) => Promise<void>
) => {
  let current = from;
  while (current <= to) {
    const concurrentTasks = [];
    for (let i = 0; i < concurrency && current <= to; i++) {
      concurrentTasks.push(
        api.rpc.chain.getBlockHash(current++).then((hash) => getBlockDetails(api, hash))
      );
    }
    const blocksDetails = await Promise.all(concurrentTasks);
    for (const blockDetails of blocksDetails) {
      await callBack(blockDetails);
    }
  }
};

export interface RealtimeBlockDetails extends BlockDetails {
  elapsedMilliSecs: number;
  pendingTxs: Extrinsic[];
}

export const listenBlocks = async (
  api: ApiPromise,
  finalized: boolean,
  callBack: (blockDetails: RealtimeBlockDetails) => Promise<void>
) => {
  let latestBlockTime = 0;
  try {
    latestBlockTime = (
      await api.query.timestamp.now.at((await api.rpc.chain.getBlock()).block.header.parentHash)
    ).toNumber();
  } catch (e) {
    // This can happen if you start at genesis block
    latestBlockTime = 0;
  }
  const call = finalized ? api.rpc.chain.subscribeFinalizedHeads : api.rpc.chain.subscribeNewHeads;
  const unsubHeads = await call(async (lastHeader) => {
    const [blockDetails, pendingTxs] = await Promise.all([
      getBlockDetails(api, lastHeader.hash),
      api.rpc.author.pendingExtrinsics(),
    ]);
    callBack({
      ...blockDetails,
      pendingTxs,
      elapsedMilliSecs: blockDetails.blockTime - latestBlockTime,
    });
    latestBlockTime = blockDetails.blockTime;
  });
  return unsubHeads;
};

export const listenBestBlocks = async (
  api: ApiPromise,
  callBack: (blockDetails: RealtimeBlockDetails) => Promise<void>
) => {
  listenBlocks(api, false, callBack);
};

export const listenFinalizedBlocks = async (
  api: ApiPromise,
  callBack: (blockDetails: RealtimeBlockDetails) => Promise<void>
) => {
  listenBlocks(api, true, callBack);
};

export function printDetails(
  block: Block,
  pendingTxs,
  elapsedMilliSecs,
  weightPercentage,
  options?: { prefix?: string; suffix?: string }
) {
  const seconds = elapsedMilliSecs
    ? (Math.floor(elapsedMilliSecs / 100) / 10).toFixed(1).padStart(5, " ")
    : null;
  const secondText = elapsedMilliSecs
    ? elapsedMilliSecs > 30000
      ? chalk.red(seconds)
      : elapsedMilliSecs > 14000
      ? chalk.yellow(seconds)
      : seconds
    : null;

  const weight = weightPercentage.toFixed(2).padStart(5, " ");
  const weightText =
    weightPercentage > 60
      ? chalk.red(weight)
      : weightPercentage > 30
      ? chalk.yellow(weight)
      : weight;

  const txPool = pendingTxs ? pendingTxs.length.toString().padStart(5, " ") : null;
  const txPoolText = pendingTxs
    ? pendingTxs.length > 1000
      ? chalk.red(txPool)
      : pendingTxs.length > 100
      ? chalk.yellow(txPool)
      : txPool
    : null;

  const ext = block.extrinsics.length.toString().padStart(4, " ");
  const extText =
    block.extrinsics.length >= 40
      ? chalk.red(ext)
      : block.extrinsics.length >= 10
      ? chalk.yellow(ext)
      : block.extrinsics.length > 3
      ? chalk.green(ext)
      : ext;

  const ethTxs = block.extrinsics.filter(
    (tx) => tx.method.section == "ethereum" && tx.method.method == "transact"
  ).length;
  const eths = ethTxs.toString().padStart(4, " ");
  const evmText =
    ethTxs >= 40
      ? chalk.red(eths)
      : ethTxs >= 10
      ? chalk.yellow(eths)
      : ethTxs > 3
      ? chalk.green(eths)
      : eths;

  const authorId = block.extrinsics
    .find((tx) => tx.method.section == "authorInherent" && tx.method.method == "setAuthor")
    .args[0].toString();

  const hash = block.header.hash.toString();
  console.log(
    `${options?.prefix ? `${options.prefix} ` : ""}Block ${block.header.number
      .toString()
      .padEnd(7, " ")} [${weightText}%][Ext:${extText} < Eth:${evmText}]${
      pendingTxs ? `[Pool:${txPoolText}]` : ``
    }${elapsedMilliSecs ? `[${secondText}s]` : ""}(hash: ${hash.substring(0, 7)}...${hash.substring(
      hash.length - 4
    )})${options?.suffix ? ` ${options.suffix}` : ""} by ${authorId.substring(
      0,
      7
    )}...${authorId.substring(authorId.length - 4)}`
  );
}

export function printRealtimeBlockDetails(
  { block, pendingTxs, elapsedMilliSecs, weightPercentage }: RealtimeBlockDetails,
  options?: { prefix?: string; suffix?: string }
) {
  return printDetails(block, pendingTxs, elapsedMilliSecs, weightPercentage, options);
}

export function printBlockDetails(
  { block, weightPercentage }: BlockDetails,
  options?: { prefix?: string; suffix?: string }
) {
  return printDetails(block, null, null, weightPercentage, options);
}

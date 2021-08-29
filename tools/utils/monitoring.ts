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
  pendingTxs: Extrinsic[];
  weightPercentage: number;
}

const getBlockDetails = async (api: ApiPromise, blockHash: BlockHash) => {
  const maxBlockWeight = api.consts.system.blockWeights.maxBlock.toBigInt();
  const [{ block }, pendingTxs, records, blockTime] = await Promise.all([
    api.rpc.chain.getBlock(blockHash),
    api.rpc.author.pendingExtrinsics(),
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
    pendingTxs,
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

export interface ContinuousBlockDetails extends BlockDetails {
  elapsedMilliSecs: number;
}

export const listenBlocks = async (
  api: ApiPromise,
  callBack: (blockDetails: ContinuousBlockDetails) => Promise<void>
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
  const unsubHeads = await api.rpc.chain.subscribeNewHeads(async (lastHeader) => {
    const blockDetails = await getBlockDetails(api, lastHeader.hash);
    callBack({ ...blockDetails, elapsedMilliSecs: blockDetails.blockTime - latestBlockTime });
    latestBlockTime = blockDetails.blockTime;
  });
  return unsubHeads;
};

export function printBlockDetails(
  { block, pendingTxs, elapsedMilliSecs, weightPercentage }: ContinuousBlockDetails,
  options?: { prefix: string }
) {
  const seconds = (Math.floor(elapsedMilliSecs / 100) / 10).toFixed(1).padStart(5, " ");
  const secondText =
    elapsedMilliSecs > 30000
      ? chalk.red(seconds)
      : elapsedMilliSecs > 14000
      ? chalk.yellow(seconds)
      : seconds;

  const weight = weightPercentage.toFixed(2).padStart(5, " ");
  const weightText =
    weightPercentage > 60
      ? chalk.red(weight)
      : weightPercentage > 30
      ? chalk.yellow(weight)
      : weight;

  const txPool = pendingTxs.length.toString().padStart(5, " ");
  const txPoolText =
    pendingTxs.length > 1000
      ? chalk.red(txPool)
      : pendingTxs.length > 100
      ? chalk.yellow(txPool)
      : txPool;

  const ext = block.extrinsics.length.toString().padStart(4, " ");
  const extText =
    block.extrinsics.length >= 40
      ? chalk.red(ext)
      : block.extrinsics.length >= 10
      ? chalk.yellow(ext)
      : block.extrinsics.length > 3
      ? chalk.green(ext)
      : ext;

  console.log(
    `${options?.prefix ? `${options.prefix} ` : ""}Block ${block.header.number
      .toString()
      .padEnd(7, " ")} [${weightText}%][Ext:${extText}][Pool:${txPoolText}][${secondText}s]`
  );
}

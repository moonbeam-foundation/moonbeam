import { ApiPromise } from "@polkadot/api";
import { Extrinsic } from "@polkadot/types/interfaces";
import { Block } from "@polkadot/types/interfaces/runtime/types";
import type { TxWithEvent } from "@polkadot/api-derive/types";
import { mapExtrinsics } from "./types";
import chalk from "chalk";

export interface BlockDetails {
  block: Block;
  txWithEvents: TxWithEvent[];
  pendingTxs: Extrinsic[];
  elapsedMilliSecs: number;
  weightPercentage: number;
}

export const listenBlocks = async (
  api: ApiPromise,
  callBack: (blockDetails: BlockDetails) => void
) => {
  const maxBlockWeight = api.consts.system.blockWeights.maxBlock.toBigInt();
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
    const [{ block }, pendingTxs, records, blockTime] = await Promise.all([
      api.rpc.chain.getBlock(lastHeader.hash),
      api.rpc.author.pendingExtrinsics(),
      api.query.system.events.at(lastHeader.hash),
      api.query.timestamp.now.at(lastHeader.hash),
    ]);
    const txWithEvents = mapExtrinsics(block.extrinsics, records);
    const blockWeight = txWithEvents.reduce((totalWeight, tx, index) => {
      return totalWeight + (tx.dispatchInfo && tx.dispatchInfo.weight.toBigInt());
    }, 0n);
    callBack({
      block,
      elapsedMilliSecs: blockTime.toNumber() - latestBlockTime,
      weightPercentage: Number((blockWeight * 100n) / maxBlockWeight) / 100,
      txWithEvents,
      pendingTxs,
    });
    latestBlockTime = blockTime.toNumber();
  });
  return unsubHeads;
};

export function printBlockDetails(
  { block, pendingTxs, elapsedMilliSecs, weightPercentage }: BlockDetails,
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

  console.log(
    `${options?.prefix ? `${options.prefix} ` : ""}Block ${block.header.number
      .toString()
      .padEnd(7, " ")} [${weightText}%][Ext:${block.extrinsics.length
      .toString()
      .padStart(4, " ")}][Pool: ${txPoolText} txs][${secondText}s]`
  );
}

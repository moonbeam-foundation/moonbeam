import { ApiPromise } from "@polkadot/api";
import { Extrinsic, BlockHash, EventRecord } from "@polkadot/types/interfaces";
import { Block } from "@polkadot/types/interfaces/runtime/types";
import type { TxWithEvent } from "@polkadot/api-derive/types";
import { mapExtrinsics, TxWithEventAndFee } from "./types";
import chalk from "chalk";

export interface BlockDetails {
  block: Block;
  blockTime: number;
  records: EventRecord[];
  txWithEvents: TxWithEventAndFee[];
  weightPercentage: number;
}

const getBlockDetails = async (api: ApiPromise, blockHash: BlockHash) => {
  const maxBlockWeight = api.consts.system.blockWeights.maxBlock.toBigInt();
  const [{ block }, records, blockTime] = await Promise.all([
    api.rpc.chain.getBlock(blockHash),
    api.query.system.events.at(blockHash),
    api.query.timestamp.now.at(blockHash),
  ]);

  const fees = await Promise.all(
    block.extrinsics.map((ext) => api.rpc.payment.queryInfo(ext.toHex(), blockHash))
  );

  const txWithEvents = mapExtrinsics(block.extrinsics, records, fees);
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

export function printBlockDetails(
  blockDetails: BlockDetails | RealtimeBlockDetails,
  options?: { prefix?: string; suffix?: string },
  previousBlockDetails?: BlockDetails | RealtimeBlockDetails
) {
  let secondText = null;
  if (previousBlockDetails) {
    const elapsedMilliSecs = blockDetails.blockTime - previousBlockDetails.blockTime;
    const seconds = (Math.floor(elapsedMilliSecs / 100) / 10).toFixed(1).padStart(5, " ");
    secondText =
      elapsedMilliSecs > 30000
        ? chalk.red(seconds)
        : elapsedMilliSecs > 14000
        ? chalk.yellow(seconds)
        : seconds;
  }

  const weight = blockDetails.weightPercentage.toFixed(2).padStart(5, " ");
  const weightText =
    blockDetails.weightPercentage > 60
      ? chalk.red(weight)
      : blockDetails.weightPercentage > 30
      ? chalk.yellow(weight)
      : blockDetails.weightPercentage > 10
      ? chalk.green(weight)
      : weight;

  let txPoolText = null;
  let poolIncText = null;
  let zoomPool = null;
  if ("pendingTxs" in blockDetails) {
    const txPool = blockDetails.pendingTxs.length.toString().padStart(4, " ");
    txPoolText =
      blockDetails.pendingTxs.length > 1000
        ? chalk.red(txPool)
        : blockDetails.pendingTxs.length > 100
        ? chalk.yellow(txPool)
        : txPool;

    if (previousBlockDetails && "pendingTxs" in previousBlockDetails) {
      const newPendingHashes = previousBlockDetails.pendingTxs.map((tx) => tx.hash.toString());
      const txPoolDiff = blockDetails.pendingTxs
        .map((tx) => tx.hash.toString())
        .filter((x) => !newPendingHashes.includes(x)).length;
      const poolInc = txPoolDiff.toString().padStart(3, " ");
      poolIncText =
        txPoolDiff > 80 ? chalk.red(poolInc) : txPoolDiff > 30 ? chalk.yellow(poolInc) : poolInc;
    }
    zoomPool = blockDetails.pendingTxs
      .filter((tx) => {
        return (
          tx.method.section == "ethereum" &&
          tx.method.method == "transact" &&
          (tx.method.args[0] as any).action.isCall &&
          (tx.method.args[0] as any).action.asCall.toString().toLowerCase() ==
            "0x08716e418e68564c96b68192e985762740728018".toLowerCase()
        );
      })
      .length.toString()
      .padStart(3, " ");
  }

  const ext = blockDetails.block.extrinsics.length.toString().padStart(3, " ");
  const extText =
    blockDetails.block.extrinsics.length >= 100
      ? chalk.red(ext)
      : blockDetails.block.extrinsics.length >= 50
      ? chalk.yellow(ext)
      : blockDetails.block.extrinsics.length > 15
      ? chalk.green(ext)
      : ext;

  const ethTxs = blockDetails.block.extrinsics.filter(
    (tx) => tx.method.section == "ethereum" && tx.method.method == "transact"
  ).length;
  const eths = ethTxs.toString().padStart(3, " ");
  const evmText =
    ethTxs >= 97
      ? chalk.red(eths)
      : ethTxs >= 47
      ? chalk.yellow(eths)
      : ethTxs > 12
      ? chalk.green(eths)
      : eths;

  const fees = blockDetails.txWithEvents
    .filter(({ dispatchInfo }) => dispatchInfo.paysFee.isYes && !dispatchInfo.class.isMandatory)
    .reduce((p, { dispatchInfo, extrinsic, events, fee }) => {
      if (extrinsic.method.section == "ethereum") {
        return (
          p +
          (BigInt((extrinsic.method.args[0] as any).gasPrice) * dispatchInfo.weight.toBigInt()) /
            25000n
        );
      }
      return p + fee.partialFee.toBigInt();
    }, 0n);
  const feesTokens = Number(fees / 10n ** 15n) / 1000;
  const feesTokenTxt = feesTokens.toFixed(3).padStart(5, " ");
  const feesText =
    feesTokens >= 0.1
      ? chalk.red(feesTokenTxt)
      : feesTokens >= 0.01
      ? chalk.yellow(feesTokenTxt)
      : feesTokens >= 0.001
      ? chalk.green(feesTokenTxt)
      : feesTokenTxt;

  const extZoom = blockDetails.block.extrinsics
    .filter(
      (tx) =>
        tx.method.section == "ethereum" &&
        tx.method.method == "transact" &&
        (tx.method.args[0] as any).action.isCall &&
        (tx.method.args[0] as any).action.asCall.toString().toLowerCase() ==
          "0x08716e418e68564c96b68192e985762740728018".toLowerCase()
    )
    .length.toString()
    .padStart(3, " ");
  const authorId = blockDetails.block.extrinsics
    .find((tx) => tx.method.section == "authorInherent" && tx.method.method == "setAuthor")
    .args[0].toString();

  const hash = blockDetails.block.header.hash.toString();
  console.log(
    `${options?.prefix ? `${options.prefix} ` : ""}Block ${blockDetails.block.header.number
      .toString()
      .padEnd(
        7,
        " "
      )} [${weightText}%, ${feesText}💰][Ext:${extText}(Eth:${evmText})(Z:${extZoom})]${
      txPoolText
        ? `[Pool:${txPoolText}${poolIncText ? `(+${poolIncText})` : ""}(Z ${zoomPool})]`
        : ``
    }${secondText ? `[${secondText}s]` : ""}(hash: ${hash.substring(0, 7)}..${hash.substring(
      hash.length - 4
    )})${options?.suffix ? ` ${options.suffix}` : ""} by ${authorId.substring(
      0,
      7
    )}..${authorId.substring(authorId.length - 4)}`
  );
}

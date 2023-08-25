import "@moonbeam-network/api-augment/moonbase";
import type {
  RuntimeDispatchInfoV1,
  RuntimeDispatchInfoV2,
} from "@polkadot/types/interfaces/payment";
import { ApiPromise } from "@polkadot/api";
import {
  BlockHash,
  DispatchError,
  DispatchInfo,
  Extrinsic,
  RuntimeDispatchInfo,
} from "@polkadot/types/interfaces";
import { FrameSystemEventRecord, SpWeightsWeightV2Weight } from "@polkadot/types/lookup";
import { u32, u64, u128, Option } from "@polkadot/types";
import { expect } from "chai";

import { EXTRINSIC_BASE_WEIGHT, WEIGHT_PER_GAS } from "./constants";
import { DevTestContext } from "./setup-dev-tests";
import { rateLimiter } from "./common";
import type { Block, AccountId20 } from "@polkadot/types/interfaces/runtime/types";
import type { TxWithEvent } from "@polkadot/api-derive/types";
import type { ITuple } from "@polkadot/types-codec/types";

const debug = require("debug")("test:blocks");
export async function createAndFinalizeBlock(
  api: ApiPromise,
  parentHash?: string,
  finalize: boolean = true
): Promise<{
  duration: number;
  hash: string;
  proof_size?: number;
}> {
  const startTime: number = Date.now();

  // Faking block creation when running dev test against a real
  // parachain network. (like with forked networks)
  if (!api.rpc.engine?.createBlock) {
    const startingBlock = await api.rpc.chain.getBlock();
    let block = startingBlock;
    while (block.hash.toString() == startingBlock.hash.toString()) {
      block = await api.rpc.chain.getBlock();
      await new Promise((resolve) => setTimeout(resolve, 200));
    }
    return {
      duration: Date.now() - startTime,
      hash: block.hash.toString(),
      proof_size: 0,
    };
  }

  const block: any = parentHash
    ? await api.rpc("engine_createBlock", true, finalize, parentHash)
    : await api.rpc("engine_createBlock", true, finalize);

  return {
    duration: Date.now() - startTime,
    hash: block.hash as string,
    proof_size: block.proof_size as number,
  };
}

// Given a deposit amount, returns the amount burned (80%) and deposited to treasury (20%).
// This is meant to precisely mimic the logic in the Moonbeam runtimes where the burn amount
// is calculated and the treasury is treated as the remainder. This precision is important to
// avoid off-by-one errors.
export function calculateFeePortions(amount: bigint): { burnt: bigint; treasury: bigint } {
  const burnt = (amount * 80n) / 100n; // 20% goes to treasury
  return { burnt, treasury: amount - burnt };
}

export interface TxWithEventAndFee extends TxWithEvent {
  fee: RuntimeDispatchInfo | RuntimeDispatchInfoV1 | RuntimeDispatchInfoV2;
}

export interface BlockDetails {
  block: Block;
  txWithEvents: TxWithEventAndFee[];
}

const getBlockDetails = async (
  api: ApiPromise,
  blockHash: BlockHash | string | any
): Promise<BlockDetails> => {
  debug(`Querying ${blockHash}`);

  const [{ block }, records] = await Promise.all([
    api.rpc.chain.getBlock(blockHash),
    await (await api.at(blockHash)).query.system.events(),
  ]);

  const fees = await Promise.all(
    block.extrinsics.map(async (ext) =>
      (
        await api.at(block.header.parentHash)
      ).call.transactionPaymentApi.queryInfo(ext.toHex(), ext.encodedLength)
    )
  );

  const txWithEvents = mapExtrinsics(block.extrinsics, records, fees);

  return {
    block,
    txWithEvents,
  } as any as BlockDetails;
};

export interface BlockRangeOption {
  from: number;
  to: number;
  concurrency?: number;
}

// Explore all blocks for the given range and returns block information for each one
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

export const verifyBlockFees = async (
  context: DevTestContext,
  fromBlockNumber: number,
  toBlockNumber: number,
  expectedBalanceDiff: bigint
) => {
  const api = context.polkadotApi;
  debug(`========= Checking block ${fromBlockNumber}...${toBlockNumber}`);
  let sumBlockFees = 0n;
  let sumBlockBurnt = 0n;

  // Get from block hash and totalSupply
  const fromPreBlockHash = (await api.rpc.chain.getBlockHash(fromBlockNumber - 1)).toString();
  const fromPreSupply = (await (
    await api.at(fromPreBlockHash)
  ).query.balances.totalIssuance()) as any;
  let previousBlockHash = fromPreBlockHash;

  // Get to block hash and totalSupply
  const toBlockHash = (await api.rpc.chain.getBlockHash(toBlockNumber)).toString();
  const toSupply = (await (await api.at(toBlockHash)).query.balances.totalIssuance()) as any;

  // fetch block information for all blocks in the range
  await exploreBlockRange(
    api,
    { from: fromBlockNumber, to: toBlockNumber, concurrency: 5 },
    async (blockDetails) => {
      let blockFees = 0n;
      let blockBurnt = 0n;

      // iterate over every extrinsic
      for (const txWithEvents of blockDetails.txWithEvents) {
        let { events, extrinsic, fee } = txWithEvents;

        // This hash will only exist if the transaction was executed through ethereum.
        let ethereumAddress = "";

        if (extrinsic.method.section == "ethereum") {
          // Search for ethereum execution
          events.forEach((event) => {
            if (event.section == "ethereum" && event.method == "Executed") {
              ethereumAddress = event.data[0].toString();
            }
          });
        }

        // Payment event is submitted for substrate transactions
        let paymentEvent = events.find(
          (event) => event.section == "transactionPayment" && event.method == "TransactionFeePaid"
        );

        let txFees = 0n;
        let txBurnt = 0n;
        // For every extrinsic, iterate over every event
        // and search for ExtrinsicSuccess or ExtrinsicFailed
        for (const event of events) {
          if (
            event.section == "system" &&
            (event.method == "ExtrinsicSuccess" || event.method == "ExtrinsicFailed")
          ) {
            const dispatchInfo =
              event.method == "ExtrinsicSuccess"
                ? (event.data[0] as DispatchInfo)
                : (event.data[1] as DispatchInfo);

            // We are only interested in fee paying extrinsics:
            // Either ethereum transactions or signed extrinsics with fees (substrate tx)
            if (
              (dispatchInfo.paysFee.isYes && !extrinsic.signer.isEmpty) ||
              extrinsic.method.section == "ethereum"
            ) {
              if (extrinsic.method.section == "ethereum") {
                // For Ethereum tx we caluculate fee by first converting weight to gas
                const gasUsed = (dispatchInfo as any).weight.refTime.toBigInt() / WEIGHT_PER_GAS;
                let ethTxWrapper = extrinsic.method.args[0] as any;

                let number = blockDetails.block.header.number.toNumber();
                // The on-chain base fee used by the transaction. Aka the parent block's base fee.
                //
                // Note on 1559 fees: no matter what the user was willing to pay (maxFeePerGas),
                // the transaction fee is ultimately computed using the onchain base fee. The
                // additional tip eventually paid by the user (maxPriorityFeePerGas) is purely a
                // prioritization component: the EVM is not aware of it and thus not part of the
                // weight cost of the extrinsic.
                let baseFeePerGas = BigInt(
                  (await context.web3.eth.getBlock(number - 1)).baseFeePerGas
                );
                let priorityFee;

                // Transaction is an enum now with as many variants as supported transaction types.
                if (ethTxWrapper.isLegacy) {
                  priorityFee = ethTxWrapper.asLegacy.gasPrice.toBigInt();
                } else if (ethTxWrapper.isEip2930) {
                  priorityFee = ethTxWrapper.asEip2930.gasPrice.toBigInt();
                } else if (ethTxWrapper.isEip1559) {
                  priorityFee = ethTxWrapper.asEip1559.maxPriorityFeePerGas.toBigInt();
                }

                let effectiveTipPerGas = priorityFee - baseFeePerGas;
                if (effectiveTipPerGas < 0n) {
                  effectiveTipPerGas = 0n;
                }

                // Calculate the fees paid for base fee independently from tip fee. Both are subject
                // to 80/20 split (burn/treasury) but calculating these over the sum of the two
                // rather than independently leads to off-by-one errors.
                const baseFeesPaid = gasUsed * baseFeePerGas;
                const tipAsFeesPaid = gasUsed * effectiveTipPerGas;

                const baseFeePortions = calculateFeePortions(baseFeesPaid);
                const tipFeePortions = calculateFeePortions(tipAsFeesPaid);

                txFees += baseFeesPaid + tipAsFeesPaid;
                txBurnt += baseFeePortions.burnt;
                txBurnt += tipFeePortions.burnt;
              } else {
                // For a regular substrate tx, we use the partialFee
                const feePortions = calculateFeePortions(fee.partialFee.toBigInt());
                const tipPortions = calculateFeePortions(extrinsic.tip.toBigInt());
                txFees += fee.partialFee.toBigInt() + extrinsic.tip.toBigInt();
                txBurnt += feePortions.burnt + tipPortions.burnt;

                // verify entire substrate txn fee
                const apiAt = await context.polkadotApi.at(previousBlockHash);
                const lengthFee = (
                  (await apiAt.call.transactionPaymentApi.queryLengthToFee(
                    extrinsic.encodedLength
                  )) as any
                ).toBigInt();

                const unadjustedWeightFee = (
                  await apiAt.call.transactionPaymentApi.queryWeightToFee(
                    "refTime" in fee.weight
                      ? fee.weight
                      : {
                          refTime: fee.weight,
                          proofSize: 0n,
                        }
                  )
                ).toBigInt();
                const multiplier = await apiAt.query.transactionPayment.nextFeeMultiplier();
                const denominator = 1_000_000_000_000_000_000n;
                const weightFee = (unadjustedWeightFee * multiplier.toBigInt()) / denominator;

                const baseFee = (
                  (await apiAt.call.transactionPaymentApi.queryWeightToFee({
                    refTime: EXTRINSIC_BASE_WEIGHT,
                    proofSize: 0n,
                  })) as any
                ).toBigInt();

                const tip = extrinsic.tip.toBigInt();
                const expectedPartialFee = lengthFee + weightFee + baseFee;

                // Verify the computed fees are equal to the actual fees
                expect(expectedPartialFee).to.eq((paymentEvent.data[1] as u128).toBigInt() - tip);

                // Verify the computed fees are equal to the rpc computed fees
                expect(expectedPartialFee).to.eq(fee.partialFee.toBigInt());
              }

              blockFees += txFees;
              blockBurnt += txBurnt;

              const origin = extrinsic.signer.isEmpty
                ? ethereumAddress
                : extrinsic.signer.toString();

              // Get balance of the origin account both before and after extrinsic execution
              const fromBalance = (await (
                await api.at(previousBlockHash)
              ).query.system.account(origin)) as any;
              const toBalance = (await (
                await api.at(blockDetails.block.hash)
              ).query.system.account(origin)) as any;

              expect(txFees.toString()).to.eq(
                (
                  (((fromBalance.data.free.toBigInt() as any) -
                    toBalance.data.free.toBigInt()) as any) - expectedBalanceDiff
                ).toString()
              );
            }
          }
        }
        // Then search for Deposit event from treasury
        // This is for bug detection when the fees are not matching the expected value
        // TODO: sudo should not have treasury event
        const allDeposits = events
          .filter(
            (event) =>
              event.section == "treasury" &&
              event.method == "Deposit" &&
              extrinsic.method.section !== "sudo"
          )
          .map((event) => (event.data[0] as any).toBigInt())
          .reduce((p, v) => p + v, 0n);

        expect(
          txFees - txBurnt,
          `Desposit Amount Discrepancy!\n` +
            `    Block: #${blockDetails.block.header.number.toString()}\n` +
            `Extrinsic: ${extrinsic.method.section}.${extrinsic.method.method}\n` +
            `     Args: \n` +
            extrinsic.args.map((arg) => `          - ${arg.toString()}\n`).join("") +
            `   Events: \n` +
            events
              .map(({ data, method, section }) => `          - ${section}.${method}:: ${data}\n`)
              .join("") +
            `     fees not burnt : ${(txFees - txBurnt).toString().padStart(30, " ")}\n` +
            `       all deposits : ${allDeposits.toString().padStart(30, " ")}`
        ).to.eq(allDeposits);
      }
      sumBlockFees += blockFees;
      sumBlockBurnt += blockBurnt;
      previousBlockHash = blockDetails.block.hash.toString();
    }
  );

  expect(fromPreSupply.toBigInt() - toSupply.toBigInt()).to.eq(sumBlockBurnt);

  // Log difference in supply, we should be equal to the burnt fees
  // debug(
  //   `  supply diff: ${(fromPreSupply.toBigInt() - toSupply.toBigInt())
  //     .toString()
  //     .padStart(30, " ")}`
  // );
  // debug(`  burnt fees : ${sumBlockBurnt.toString().padStart(30, " ")}`);
  // debug(`  total fees : ${sumBlockFees.toString().padStart(30, " ")}`);
};

export const verifyLatestBlockFees = async (
  context: DevTestContext,
  expectedBalanceDiff: bigint = BigInt(0)
) => {
  const signedBlock = await context.polkadotApi.rpc.chain.getBlock();
  const blockNumber = Number(signedBlock.block.header.number);
  return verifyBlockFees(context, blockNumber, blockNumber, expectedBalanceDiff);
};

export const getBlockExtrinsic = async (
  api: ApiPromise,
  blockHash: string | BlockHash,
  section: string,
  method: string
) => {
  const apiAt = await api.at(blockHash);
  const [{ block }, records] = await Promise.all([
    api.rpc.chain.getBlock(blockHash),
    apiAt.query.system.events(),
  ]);
  const extIndex = block.extrinsics.findIndex(
    (ext) => ext.method.section == section && ext.method.method == method
  );
  const extrinsic = extIndex > -1 ? block.extrinsics[extIndex] : null;

  const events = records
    .filter(({ phase }) => phase.isApplyExtrinsic && phase.asApplyExtrinsic.eq(extIndex))
    .map(({ event }) => event);
  const resultEvent = events.find(
    (event) =>
      event.section === "system" &&
      (event.method === "ExtrinsicSuccess" || event.method === "ExtrinsicFailed")
  );
  return { block, extrinsic, events, resultEvent };
};

export async function jumpToRound(context: DevTestContext, round: number): Promise<string | null> {
  let lastBlockHash = null;
  while (true) {
    const currentRound = (
      await context.polkadotApi.query.parachainStaking.round()
    ).current.toNumber();
    if (currentRound === round) {
      return lastBlockHash;
    } else if (currentRound > round) {
      return null;
    }

    lastBlockHash = (await context.createBlock()).block.hash.toString();
  }
}

export async function jumpBlocks(context: DevTestContext, blockCount: number) {
  while (blockCount > 0) {
    (await context.createBlock()).block.hash.toString();
    blockCount--;
  }
}

export async function jumpRounds(context: DevTestContext, count: Number): Promise<string | null> {
  const round = (await context.polkadotApi.query.parachainStaking.round()).current
    .addn(count.valueOf())
    .toNumber();

  return jumpToRound(context, round);
}

export const getBlockTime = (signedBlock: any) =>
  signedBlock.block.extrinsics
    .find((item) => item.method.section == "timestamp")
    .method.args[0].toNumber();

export const checkBlockFinalized = async (api: ApiPromise, number: number) => {
  return {
    number,
    finalized: (await api.rpc.moon.isBlockFinalized(await api.rpc.chain.getBlockHash(number)))
      .isTrue,
  };
};

// Determine if the block range intersects with an upgrade event
export const checkTimeSliceForUpgrades = async (
  api: ApiPromise,
  blockNumbers: number[],
  currentVersion: u32
) => {
  const apiAt = await api.at(await api.rpc.chain.getBlockHash(blockNumbers[0]));
  const onChainRt = (await apiAt.query.system.lastRuntimeUpgrade()).unwrap().specVersion;
  return { result: !onChainRt.eq(currentVersion), specVersion: onChainRt };
};

const fetchBlockTime = async (api: ApiPromise, blockNum: number) => {
  const hash = await api.rpc.chain.getBlockHash(blockNum);
  const block = await api.rpc.chain.getBlock(hash);
  return getBlockTime(block);
};

export const fetchHistoricBlockNum = async (
  api: ApiPromise,
  blockNumber: number,
  targetTime: number
) => {
  if (blockNumber <= 1) {
    return 1;
  }
  const time = await fetchBlockTime(api, blockNumber);

  if (time <= targetTime) {
    return blockNumber;
  }

  return fetchHistoricBlockNum(
    api,
    blockNumber - Math.ceil((time - targetTime) / 30_000),
    targetTime
  );
};

export const getBlockArray = async (api: ApiPromise, timePeriod: number) => {
  /**  
  @brief Returns an sequential array of block numbers from a given period of time in the past
  @param api Connected ApiPromise to perform queries on
  @param timePeriod Moment in the past to search until
  @param limiter Bottleneck rate limiter to throttle requests
  */

  const limiter = rateLimiter();
  const finalizedHead = await limiter.schedule(() => api.rpc.chain.getFinalizedHead());
  const signedBlock = await limiter.schedule(() => api.rpc.chain.getBlock(finalizedHead));

  const lastBlockNumber = signedBlock.block.header.number.toNumber();
  const lastBlockTime = getBlockTime(signedBlock);

  const firstBlockTime = lastBlockTime - timePeriod;
  debug(`Searching for the block at: ${new Date(firstBlockTime)}`);
  const firstBlockNumber = (await limiter.wrap(fetchHistoricBlockNum)(
    api,
    lastBlockNumber,
    firstBlockTime
  )) as number;

  const length = lastBlockNumber - firstBlockNumber;
  return Array.from({ length }, (_, i) => firstBlockNumber + i);
};

export function extractWeight(
  weightV1OrV2: u64 | Option<u64> | SpWeightsWeightV2Weight | Option<SpWeightsWeightV2Weight>
) {
  if ("isSome" in weightV1OrV2) {
    const weight = weightV1OrV2.unwrap();
    if ("refTime" in weight) {
      return weight.refTime.unwrap();
    }
    return weight;
  }
  if ("refTime" in weightV1OrV2) {
    return weightV1OrV2.refTime.unwrap();
  }
  return weightV1OrV2;
}

export function extractPreimageDeposit(
  request:
    | Option<ITuple<[AccountId20, u128]>>
    | {
        readonly deposit: ITuple<[AccountId20, u128]>;
        readonly len: u32;
      }
    | {
        readonly deposit: Option<ITuple<[AccountId20, u128]>>;
        readonly count: u32;
        readonly len: Option<u32>;
      }
) {
  const deposit = "deposit" in request ? request.deposit : request;
  if ("isSome" in deposit && deposit.isSome) {
    return {
      accountId: deposit.unwrap()[0].toHex(),
      amount: deposit.unwrap()[1],
    };
  }

  if (deposit.isEmpty) {
    return { accountId: "", amount: 0n };
  }

  return {
    accountId: deposit[0].toHex(),
    amount: deposit[1],
  };
}

export function mapExtrinsics(
  extrinsics: Extrinsic[],
  records: FrameSystemEventRecord[],
  fees?: RuntimeDispatchInfo[] | RuntimeDispatchInfoV1[]
): TxWithEventAndFee[] {
  return extrinsics.map((extrinsic, index): TxWithEventAndFee => {
    let dispatchError: DispatchError | undefined;
    let dispatchInfo: DispatchInfo | undefined;

    const events = records
      .filter(({ phase }) => phase.isApplyExtrinsic && phase.asApplyExtrinsic.eq(index))
      .map(({ event }) => {
        if (event.section === "system") {
          if (event.method === "ExtrinsicSuccess") {
            dispatchInfo = event.data[0] as any as DispatchInfo;
          } else if (event.method === "ExtrinsicFailed") {
            dispatchError = event.data[0] as any as DispatchError;
            dispatchInfo = event.data[1] as any as DispatchInfo;
          }
        }

        return event as any;
      });

    return { dispatchError, dispatchInfo, events, extrinsic, fee: fees ? fees[index] : undefined };
  });
}

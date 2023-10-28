import "@moonbeam-network/api-augment/moonbase";
import { DevModeContext, expect } from "@moonwall/cli";
import {
  BlockRangeOption,
  EXTRINSIC_BASE_WEIGHT,
  WEIGHT_PER_GAS,
  calculateFeePortions,
  mapExtrinsics,
} from "@moonwall/util";
import { ApiPromise } from "@polkadot/api";
import type { TxWithEvent } from "@polkadot/api-derive/types";
import { Option, u128, u32 } from "@polkadot/types";
import type { ITuple } from "@polkadot/types-codec/types";
import { BlockHash, DispatchInfo, RuntimeDispatchInfo } from "@polkadot/types/interfaces";
import type { RuntimeDispatchInfoV1 } from "@polkadot/types/interfaces/payment";
import type { AccountId20, Block } from "@polkadot/types/interfaces/runtime/types";
import chalk from "chalk";
import { Debugger } from "debug";
import Debug from "debug";
const debug = Debug("test:blocks");

// Given a deposit amount, returns the amount burned (80%) and deposited to treasury (20%).
// This is meant to precisely mimic the logic in the Moonbeam runtimes where the burn amount
// is calculated and the treasury is treated as the remainder. This precision is important to
// avoid off-by-one errors.

export interface TxWithEventAndFee extends TxWithEvent {
  fee: RuntimeDispatchInfo | RuntimeDispatchInfoV1;
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

// Explore all blocks for the given range and returns block information for each one
// fromBlockNumber and toBlockNumber included
export const exploreBlockRange = async (
  api: ApiPromise,
  { from, to, concurrency = 1 }: BlockRangeOption,
  callBack: (blockDetails: BlockDetails) => Promise<void>
) => {
  let current = from;
  while (current <= to) {
    const concurrentTasks: any[] = [];
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
  context: DevModeContext,
  fromBlockNumber: number,
  toBlockNumber: number,
  expectedBalanceDiff: bigint
) => {
  const api = context.polkadotJs();
  debug(`========= Checking block ${fromBlockNumber}...${toBlockNumber}`);
  // let sumBlockFees = 0n;
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
      // let blockFees = 0n;
      let blockBurnt = 0n;

      // iterate over every extrinsic
      for (const txWithEvents of blockDetails.txWithEvents) {
        const { events, extrinsic, fee } = txWithEvents;

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
        const paymentEvent = events.find(
          (event) => event.section == "transactionPayment" && event.method == "TransactionFeePaid"
        );

        let txFees = 0n;
        let txBurnt = 0n;
        // For every extrinsic, iterate over every event
        // and search for ExtrinsicSuccess or ExtrinsicFailed
        for (const event of events) {
          if (
            api.events.system.ExtrinsicSuccess.is(event) ||
            api.events.system.ExtrinsicFailed.is(event)
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
                const ethTxWrapper = extrinsic.method.args[0] as any;

                const number = blockDetails.block.header.number.toNumber();
                // The on-chain base fee used by the transaction. Aka the parent block's base fee.
                //
                // Note on 1559 fees: no matter what the user was willing to pay (maxFeePerGas),
                // the transaction fee is ultimately computed using the onchain base fee. The
                // additional tip eventually paid by the user (maxPriorityFeePerGas) is purely a
                // prioritization component: the EVM is not aware of it and thus not part of the
                // weight cost of the extrinsic.

                // let baseFeePerGas = BigInt(
                //   (await context.web3().eth.getBlock(number - 1)).baseFeePerGas!
                // );
                const baseFeePerGas = (
                  await context.viem().getBlock({ blockNumber: BigInt(number - 1) })
                ).baseFeePerGas!;
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
                const apiAt = await context.polkadotJs().at(previousBlockHash);
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

                // const tip = extrinsic.tip.toBigInt();
                const expectedPartialFee = lengthFee + weightFee + baseFee;

                // Verify the computed fees are equal to the actual fees
                expect(expectedPartialFee).to.eq((paymentEvent!.data[1] as u128).toBigInt());

                // Verify the computed fees are equal to the rpc computed fees
                expect(expectedPartialFee).to.eq(fee.partialFee.toBigInt());
              }

              // blockFees += txFees;
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
      // sumBlockFees += blockFees;
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
  context: DevModeContext,
  expectedBalanceDiff: bigint = BigInt(0)
) => {
  const signedBlock = await context.polkadotJs().rpc.chain.getBlock();
  const blockNumber = Number(signedBlock.block.header.number);
  return verifyBlockFees(context, blockNumber, blockNumber, expectedBalanceDiff);
};

export async function jumpToRound(context: DevModeContext, round: number): Promise<string | null> {
  let lastBlockHash = "";
  for (;;) {
    const currentRound = (
      await context.polkadotJs().query.parachainStaking.round()
    ).current.toNumber();
    if (currentRound === round) {
      return lastBlockHash;
    } else if (currentRound > round) {
      return null;
    }

    lastBlockHash = (await context.createBlock()).block.hash.toString();
  }
}

export async function jumpBlocks(context: DevModeContext, blockCount: number) {
  while (blockCount > 0) {
    (await context.createBlock()).block.hash.toString();
    blockCount--;
  }
}

export async function jumpRounds(context: DevModeContext, count: number): Promise<string | null> {
  const round = (await context.polkadotJs().query.parachainStaking.round()).current
    .addn(count.valueOf())
    .toNumber();

  return jumpToRound(context, round);
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
  } else if ("isNone" in deposit && deposit.isNone) {
    return undefined;
  }

  return {
    accountId: (deposit as any)[0].toHex(),
    amount: (deposit as any)[1],
  };
}

export async function countExtrinsics(
  context: DevModeContext,
  method: string,
  logger: Debugger
): Promise<[number, number, number]> {
  const block = await context.polkadotJs().rpc.chain.getBlock();
  const extrinsicCount = block.block.extrinsics.reduce(
    (acc, ext) =>
      acc + (ext.method.section === "parachainStaking" && ext.method.method === method ? 1 : 0),
    0
  );

  const maxBlockWeights = context.polkadotJs().consts.system.blockWeights;
  const blockWeights = await context.polkadotJs().query.system.blockWeight();

  const weightUtil =
    blockWeights.normal.refTime.toNumber() /
    maxBlockWeights.perClass.normal.maxTotal.unwrap().refTime.toNumber();
  const proofUtil =
    blockWeights.normal.proofSize.toNumber() /
    maxBlockWeights.perClass.normal.maxTotal.unwrap().proofSize.toNumber();

  logger(
    `  ${chalk.yellow("○")} ${chalk.gray(method)} max ${chalk.green(
      extrinsicCount
    )} per block (w: ${(weightUtil * 100).toFixed(1)}%, p: ${(proofUtil * 100).toFixed(1)}%)`
  );

  return [extrinsicCount, weightUtil, proofUtil];
}

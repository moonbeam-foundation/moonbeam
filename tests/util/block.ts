import { ApiPromise } from "@polkadot/api";
import {
  BlockHash,
  DispatchError,
  DispatchInfo,
  EventRecord,
  Extrinsic,
  RuntimeDispatchInfo,
} from "@polkadot/types/interfaces";
import type { Block } from "@polkadot/types/interfaces/runtime/types";
import type { TxWithEvent } from "@polkadot/api-derive/types";
import Debug from "debug";
import { WEIGHT_PER_GAS } from "./constants";
const debug = Debug("blocks");

export async function createAndFinalizeBlock(
  api: ApiPromise,
  parentHash?: BlockHash,
  finalize: boolean = true
): Promise<{
  duration: number;
  hash: BlockHash;
}> {
  const startTime: number = Date.now();
  let hash = undefined;
  try {
    if (parentHash == undefined) {
      hash = (await api.rpc.engine.createBlock(true, finalize)).toJSON()["hash"];
    } else {
      hash = (await api.rpc.engine.createBlock(true, finalize, parentHash)).toJSON()["hash"];
    }
  } catch (e) {
    console.log("ERROR DURING BLOCK FINALIZATION", e);
  }

  return {
    duration: Date.now() - startTime,
    hash,
  };
}

export interface TxWithEventAndFee extends TxWithEvent {
  fee: RuntimeDispatchInfo;
}

export interface BlockDetails {
  block: Block;
  // authorName: string;
  // blockTime: number;
  // records: EventRecord[];
  txWithEvents: TxWithEventAndFee[];
  // weightPercentage: number;
}

export function mapExtrinsics(
  extrinsics: Extrinsic[],
  records: EventRecord[],
  fees?: RuntimeDispatchInfo[]
): TxWithEventAndFee[] {
  return extrinsics.map((extrinsic, index): TxWithEventAndFee => {
    let dispatchError: DispatchError | undefined;
    let dispatchInfo: DispatchInfo | undefined;

    const events = records
      .filter(({ phase }) => phase.isApplyExtrinsic && phase.asApplyExtrinsic.eq(index))
      .map(({ event }) => {
        if (event.section === "system") {
          if (event.method === "ExtrinsicSuccess") {
            dispatchInfo = event.data[0] as DispatchInfo;
          } else if (event.method === "ExtrinsicFailed") {
            dispatchError = event.data[0] as DispatchError;
            dispatchInfo = event.data[1] as DispatchInfo;
          }
        }

        return event;
      });

    return { dispatchError, dispatchInfo, events, extrinsic, fee: fees ? fees[index] : undefined };
  });
}

const getBlockDetails = async (api: ApiPromise, blockHash: BlockHash): Promise<BlockDetails> => {
  debug(`Querying ${blockHash}`);
  const maxBlockWeight = api.consts.system.blockWeights.maxBlock.toBigInt();
  const [{ block }, records, blockTime] = await Promise.all([
    api.rpc.chain.getBlock(blockHash),
    api.query.system.events.at(blockHash),
    api.query.timestamp.now.at(blockHash),
  ]);

  const authorId = block.extrinsics
    .find((tx) => tx.method.section == "authorInherent" && tx.method.method == "setAuthor")
    .args[0].toString();

  // const [fees, authorName] = await Promise.all([
  //   Promise.all(
  //     block.extrinsics.map((ext) => api.rpc.payment.queryInfo(ext.toHex(), block.header.parentHash))
  //   ),
  //   getAuthorIdentity(api, authorId),
  // ]);
  const fees = await Promise.all(
    block.extrinsics.map((ext) => api.rpc.payment.queryInfo(ext.toHex(), block.header.parentHash))
  );

  const txWithEvents = mapExtrinsics(block.extrinsics, records, fees);
  const blockWeight = txWithEvents.reduce((totalWeight, tx, index) => {
    return totalWeight + (tx.dispatchInfo && tx.dispatchInfo.weight.toBigInt());
  }, 0n);
  return {
    block,
    // authorName,
    // blockTime: blockTime.toNumber(),
    // weightPercentage: Number((blockWeight * 10000n) / maxBlockWeight) / 100,
    txWithEvents,
    //  records,
  }; //as BlockDetails;
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
  api: ApiPromise,
  fromBlockNumber: number,
  toBlockNumber: number,
  expect,
  expectedBalanceDiff: bigint
) => {
  // Set to and from block numbers
  // const toBlockNumber = argv.to || (await api.rpc.chain.getBlock()).block.header.number.toNumber();
  // const fromBlockNumber = argv.from || toBlockNumber;

  console.log(`========= Checking block ${fromBlockNumber}...${toBlockNumber}`);
  let sumBlockFees = 0n;
  let sumBlockBurnt = 0n;
  let blockCount = 0;

  // Get from block hash and totalSupply
  const fromPreBlockHash = (await api.rpc.chain.getBlockHash(fromBlockNumber - 1)).toString();
  const fromPreSupply = await (await api.at(fromPreBlockHash)).query.balances.totalIssuance();
  let previousBlockHash = fromPreBlockHash;

  // Get to block hash and totalSupply
  const toBlockHash = (await api.rpc.chain.getBlockHash(toBlockNumber)).toString();
  const toSupply = await (await api.at(toBlockHash)).query.balances.totalIssuance();

  // fetch block information for all blocks in the range
  await exploreBlockRange(
    api,
    { from: fromBlockNumber, to: toBlockNumber, concurrency: 5 },
    async (blockDetails) => {
      blockCount++;
      let blockFees = 0n;
      let blockBurnt = 0n;

      // iterate over every extrinsic
      for (const { events, extrinsic, fee } of blockDetails.txWithEvents) {
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

        let txFees = 0n;
        let txBurnt = 0n;

        // For every extrinsic, iterate over every event and search for ExtrinsicSuccess or ExtrinsicFailed
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
              dispatchInfo.paysFee.isYes &&
              (!extrinsic.signer.isEmpty || extrinsic.method.section == "ethereum")
            ) {
              if (extrinsic.method.section == "ethereum") {
                // For Ethereum tx we caluculate fee by first converting weight to gas
                const gasFee = dispatchInfo.weight.toBigInt() / WEIGHT_PER_GAS;
                // And then multiplying by gasPrice
                txFees = gasFee * (extrinsic.method.args[0] as any).gasPrice.toBigInt();
              } else {
                // For a regular substrate tx, we use the partialFee
                txFees = fee.partialFee.toBigInt();
              }
              txBurnt += (txFees * 80n) / 100n; // 20% goes to treasury

              blockFees += txFees;
              blockBurnt += txBurnt;

              const origin = extrinsic.signer.isEmpty
                ? ethereumAddress
                : extrinsic.signer.toString();

              // Get balance of the origin account both before and after extrinsic execution
              const fromBalance = await (
                await api.at(previousBlockHash)
              ).query.system.account(origin);
              const toBalance = await (
                await api.at(blockDetails.block.hash)
              ).query.system.account(origin);

              console.log("txFees.toString()", txFees.toString());
              console.log(
                "(toBalance.data.free.toBigInt() - fromBalance.data.free.toBigInt()).toString()",
                (toBalance.data.free.toBigInt() - fromBalance.data.free.toBigInt()).toString()
              );
              expect(txFees.toString()).to.eq(
                (
                  fromBalance.data.free.toBigInt() -
                  toBalance.data.free.toBigInt() +
                  expectedBalanceDiff
                ).toString()
              );

              // Verbose option will display tx fee and balance change for each extrinsic
              // if (argv.verbose) {
              //   console.log(
              //     ` ${extrinsic.method.section == "ethereum" ? "[Eth]" : "[Sub]"}${
              //       event.method == "ExtrinsicSuccess" ? "(✔)" : "(X)"
              //     }${origin.toString()}: ${txFees.toString().padStart(19, " ")} (${printMOVRs(
              //       txFees,
              //       5
              //     )} MOVR) (Balance diff: ${(
              //       toBalance.data.free.toBigInt() - fromBalance.data.free.toBigInt()
              //     )
              //       .toString()
              //       .padStart(20, " ")})(${printMOVRs(
              //       toBalance.data.free.toBigInt() - fromBalance.data.free.toBigInt(),
              //       5
              //     )} MOVR)`
              //   );
              // }
            }
          }
        }
        // Then search for Deposit event from treasury
        // This is for bug detection when the fees are not matching the expected value
        for (const event of events) {
          if (event.section == "treasury" && event.method == "Deposit") {
            const deposit = (event.data[0] as any).toBigInt();
            // Compare deposit event amont to what should have been sent to deposit (if they don't match, which is not a desired behavior)
            expect(txFees - txBurnt).to.eq(deposit);
            if (txFees - txBurnt !== deposit) {
              console.log("Desposit Amount Discrepancy!");
              console.log(`fees not burnt : ${(txFees - txBurnt).toString().padStart(30, " ")}`);
              console.log(`       deposit : ${deposit.toString().padStart(30, " ")}`);
            }
          }
        }
      }
      sumBlockFees += blockFees;
      sumBlockBurnt += blockBurnt;
      // console.log(`#${blockDetails.block.header.number} Fees : ${printMOVRs(blockFees, 4)} MOVRs`);
      previousBlockHash = blockDetails.block.hash.toString();
    }
  );
  // Print total and average for the block range
  // console.log(
  //   `Total blocks : ${blockCount}, ${printMOVRs(
  //     sumBlockFees / BigInt(blockCount),
  //     4
  //   )}/block, ${printMOVRs(sumBlockFees, 4)} Total`
  // );
  expect(fromPreSupply.toBigInt() - toSupply.toBigInt()).to.eq(sumBlockBurnt);

  // Log difference in supply, we should be equal to the burnt fees
  console.log(
    `  supply diff: ${(fromPreSupply.toBigInt() - toSupply.toBigInt())
      .toString()
      .padStart(30, " ")}`
  );
  console.log(`  burnt fees : ${sumBlockBurnt.toString().padStart(30, " ")}`);
  console.log(`  total fees : ${sumBlockFees.toString().padStart(30, " ")}`);
};

export const verifyLatestBlockFees = async (
  api: ApiPromise,
  expect,
  expectedBalanceDiff: bigint = BigInt(0)
) => {
  const signedBlock = await api.rpc.chain.getBlock();
  const blockNumber = Number(signedBlock.block.header.number);
  return verifyBlockFees(api, blockNumber, blockNumber, expect, expectedBalanceDiff);
};

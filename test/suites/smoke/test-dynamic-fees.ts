import "@moonbeam-network/api-augment/moonbase";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { WEIGHT_PER_GAS, getBlockArray } from "@moonwall/util";
import type { ApiPromise } from "@polkadot/api";
import type { GenericExtrinsic } from "@polkadot/types";
import type { u128, u32 } from "@polkadot/types-codec";
import type {
  EthereumBlock,
  EthereumReceiptReceiptV3,
  FpRpcTransactionStatus,
  FrameSupportDispatchPerDispatchClassWeight,
  FrameSystemEventRecord,
  SpWeightsWeightV2Weight,
} from "@polkadot/types/lookup";
import type { AnyTuple } from "@polkadot/types/types";
import { ethers } from "ethers";
import { checkTimeSliceForUpgrades, rateLimiter, RUNTIME_CONSTANTS } from "../../helpers";
import type { DispatchInfo } from "@polkadot/types/interfaces";
import type { Perbill } from "@polkadot/types/interfaces/runtime";
import Debug from "debug";

const debug = Debug("dynamic-fees");

const timePeriod = process.env.TIME_PERIOD ? Number(process.env.TIME_PERIOD) : 2 * 60 * 60 * 1000;
const timeout = Math.floor(timePeriod / 12); // 2 hour -> 10 minute timeout
const limiter = rateLimiter();
const hours = (timePeriod / (1000 * 60 * 60)).toFixed(2);
const atBlock = process.env.AT_BLOCK ? Number(process.env.AT_BLOCK) : -1;

type BlockFilteredRecord = {
  blockNum: number;
  nextFeeMultiplier: u128;
  ethBlock: EthereumBlock;
  extrinsics: GenericExtrinsic<AnyTuple>[];
  ethersTransactionsFees: bigint[];
  baseFeePerGasInGwei: string;
  transactionStatuses: FpRpcTransactionStatus[];
  weights: FrameSupportDispatchPerDispatchClassWeight;
  events: FrameSystemEventRecord[];
  receipts: EthereumReceiptReceiptV3[];
  normalizedRefTime: Perbill;
  normalizedProofSize: Perbill;
  limitingDimension: string;
  targetWeight: bigint;
  blockWeightValue: bigint;
  minMultiplier: bigint;
  maxMultiplier: bigint;
};

enum Change {
  Increased = "INCREASED",
  Decreased = "DECREASED",
  Unchanged = "UNCHANGED",
  Unknown = "UNKNOWN",
  Invalid = "INVALID",
}

describeSuite({
  id: "S06",
  title: `Dynamic fees in past ${hours} hours should be correct`,
  foundationMethods: "read_only",
  testCases: ({ context, it, log }) => {
    let blockData: BlockFilteredRecord[];
    // includes blockData and also the previous block, needed for fee change computation.
    let allBlocks: BlockFilteredRecord[];
    let runtime: "MOONRIVER" | "MOONBEAM" | "MOONBASE";
    let specVersion: u32;
    let paraApi: ApiPromise;
    let skipAll = false;
    let targetFillPermill: bigint;

    const checkMultiplier = (prevBlock: BlockFilteredRecord, curr: u128) => {
      if (!prevBlock) {
        return Change.Unknown;
      }
      const prev = prevBlock.nextFeeMultiplier;
      switch (true) {
        case prev.lt(curr):
          return Change.Increased;
        case prev.gt(curr):
          return Change.Decreased;
        case prev.eq(curr):
          return Change.Unchanged;
        default:
          return Change.Invalid;
      }
    };

    const isChangeDirectionValid = (blockRecord: BlockFilteredRecord, change: Change) => {
      const { blockNum, blockWeightValue, targetWeight, limitingDimension } = blockRecord;
      const feeMultiplier = blockRecord.nextFeeMultiplier.toBigInt();
      const minMultiplier = blockRecord.minMultiplier;
      const maxMultiplier = blockRecord.maxMultiplier;

      switch (true) {
        case blockWeightValue >= targetWeight && change === Change.Increased:
          return true;
        case blockWeightValue >= targetWeight &&
          change === Change.Unchanged &&
          feeMultiplier === maxMultiplier:
          return true;
        case blockWeightValue < targetWeight && change === Change.Decreased:
          return true;
        case blockWeightValue < targetWeight &&
          change === Change.Unchanged &&
          feeMultiplier === minMultiplier:
          return true;
        case blockWeightValue === targetWeight && change === Change.Unchanged:
          return true;

        case change === Change.Unknown:
          return true;
        case blockWeightValue !== targetWeight && change === Change.Unchanged:
          log(
            `Note: Block #${blockNum} has UNCHANGED multiplier when it should ${blockWeightValue >= targetWeight ? "increase" : "decrease"} with limiting_demension: ${limitingDimension}, block_weight: ${blockWeightValue}, target_weight: ${targetWeight}`
          );
          log(
            `  Current multiplier: ${feeMultiplier}, MIN_FEE_MULTIPLIER: ${minMultiplier}, MAX_FEE_MULTIPLIER: ${maxMultiplier}`
          );
          return false;
        default:
          return false;
      }
    };

    beforeAll(async function () {
      paraApi = context.polkadotJs("para");

      const blockNumArray = atBlock > 0 ? [atBlock] : await getBlockArray(paraApi, timePeriod);
      // Retrieves the block before the first block of the data. This is used later to compute
      // the fee changes of the first block of the data.
      const previousBlockNumber = Math.max(blockNumArray[0] - 1, 0);

      // Retrieves version of the last block to check.
      const endsAtBlockHash = await paraApi.rpc.chain.getBlockHash(
        blockNumArray[blockNumArray.length - 1]
      );
      const version = (await paraApi.at(endsAtBlockHash)).consts.system.version;
      specVersion = version.specVersion;
      const specName = version.specName;
      runtime = specName.toUpperCase() as any;

      targetFillPermill = RUNTIME_CONSTANTS[runtime].TARGET_FILL_PERMILL.get(
        specVersion.toNumber()
      );

      log(`Target block fullness: ${Number(targetFillPermill) / 10_000}%`);
      log(
        `Collecting ${hours} hours worth of data ` +
          `[from #${blockNumArray[0]} ` +
          `to #${blockNumArray[blockNumArray.length - 1]}] ` +
          `(${blockNumArray.length} blocks, RT${specVersion.toNumber()})`
      );

      const getBlockData = async (blockNum: number) => {
        const blockHash = await paraApi.rpc.chain.getBlockHash(blockNum);
        const signedBlock = await paraApi.rpc.chain.getBlock(blockHash);
        const apiAt = await paraApi.at(blockHash);
        const ethBlock = (await apiAt.query.ethereum.currentBlock()).unwrapOrDefault();
        const ethersBlock = await context.ethers().provider!.getBlock(blockNum);
        const transactionStatuses = (
          await apiAt.query.ethereum.currentTransactionStatuses()
        ).unwrapOrDefault();
        const ethersTransactionsFees = await Promise.all(
          ethersBlock!.transactions.map(
            async (hash) => (await context.ethers().provider!.getTransactionReceipt(hash))!.gasUsed
          )
        );
        const weights = await apiAt.query.system.blockWeight();
        const receipts = (await apiAt.query.ethereum.currentReceipts()).unwrapOr([]);
        const events = await apiAt.query.system.events();
        const nextFeeMultiplier = await apiAt.query.transactionPayment.nextFeeMultiplier();

        // The following section mirrors the logic in the `TargetedFeeAdjustment` implementation in the pallet-transaction-payment.
        //
        // In the Rust implementation, the fee adjustment algorithm works by:
        // 1. Getting the current block weights for normal dispatch class
        // 2. Normalizing the dimensions (ref_time and proof_size) to compare them
        // 3. Determining which dimension is the limiting factor
        // 4. Calculating the target weight based on the target block fullness
        // 5. Comparing the current block weight to the target to determine if fees should increase or decrease
        const blockWeights = apiAt.consts.system.blockWeights;
        const normalMaxWeight = blockWeights.perClass.normal.maxTotal.unwrap();
        const normalBlockWeight = weights.normal;

        // Calculate normalized ref time and proof size dimensions based on perbill type (parts per billion)
        const BILL = 1_000_000_000n;
        const refTimeRatio =
          (normalBlockWeight.refTime.unwrap().toBigInt() * BILL) /
          normalMaxWeight.refTime.unwrap().toBigInt();
        const normalizedRefTime = apiAt.registry.createType("Perbill", refTimeRatio);

        const proofSizeRatio =
          (normalBlockWeight.proofSize.unwrap().toBigInt() * BILL) /
          normalMaxWeight.proofSize.unwrap().toBigInt();
        const normalizedProofSize = apiAt.registry.createType("Perbill", proofSizeRatio);

        // The limiting dimension is the one that has the higher normalized value (higher percentage of max).
        // If proof_size is using a higher percentage of its maximum than ref_time, then proof_size is the
        // limiting dimension, and vice versa.
        const isRefTimeLimiting = normalizedProofSize.lt(normalizedRefTime);
        const limitingDimension = isRefTimeLimiting ? "ref_time" : "proof_size";

        const normalLimitingDimension = isRefTimeLimiting
          ? normalBlockWeight.refTime.unwrap().toBigInt()
          : normalBlockWeight.proofSize.unwrap().toBigInt();

        const maxLimitingDimension = isRefTimeLimiting
          ? normalMaxWeight.refTime.unwrap().toBigInt()
          : normalMaxWeight.proofSize.unwrap().toBigInt();

        // Represents the desired block fullness as a permill (parts per million).
        // We divide by 1_000_000n to convert from permill to a ratio.
        const targetWeight = (targetFillPermill * maxLimitingDimension) / 1_000_000n;
        const blockWeightValue = normalLimitingDimension;

        debug(`Block #${blockNum}`);
        debug(`  Limiting dimension: ${limitingDimension}`);
        debug(
          `  Normalized ref_time: ${(Number(refTimeRatio) / 10_000_000).toFixed(2)}%, proof_size: ${(Number(proofSizeRatio) / 10_000_000).toFixed(2)}%`
        );
        debug(`  Target weight: ${targetWeight}, Block weight: ${blockWeightValue}\n`);

        return {
          blockNum,
          nextFeeMultiplier,
          ethBlock,
          ethersTransactionsFees,
          transactionStatuses,
          extrinsics: signedBlock.block.extrinsics,
          baseFeePerGasInGwei: ethers.formatUnits(ethersBlock!.baseFeePerGas!, "gwei"),
          weights,
          receipts,
          events,
          normalizedRefTime,
          normalizedProofSize,
          limitingDimension,
          targetWeight,
          blockWeightValue,
          minMultiplier: RUNTIME_CONSTANTS[runtime].MIN_FEE_MULTIPLIER,
          maxMultiplier: RUNTIME_CONSTANTS[runtime].MAX_FEE_MULTIPLIER,
        };
      };

      // Determine if the block range intersects with an upgrade event
      const { result, specVersion: onChainRt } = await checkTimeSliceForUpgrades(
        paraApi,
        blockNumArray,
        specVersion
      );
      if (result) {
        log(
          `Time slice of blocks intersects with upgrade ` +
            `from RT ${onChainRt} to RT ${specVersion}, skipping tests.`
        );
        skipAll = true;
      }

      blockData = await Promise.all(
        blockNumArray.map((num) => limiter.schedule(() => getBlockData(num)))
      );
      allBlocks =
        previousBlockNumber > 0
          ? [await getBlockData(previousBlockNumber), ...blockData]
          : blockData;
    }, timeout);

    it({
      id: "C100",
      title: "Block utilization by weight corresponds to fee multiplier",
      timeout: 30000,
      test: function () {
        if (skipAll) {
          log("Skipping test suite due to runtime version");
          return;
        }
        const enriched = blockData.map((blockRecord) => {
          const {
            blockNum,
            nextFeeMultiplier,
            blockWeightValue,
            targetWeight,
            limitingDimension,
            normalizedRefTime,
            normalizedProofSize,
          } = blockRecord;

          const prevBlock = allBlocks.find((blockDatum) => blockDatum.blockNum === blockNum - 1)!;
          const change = checkMultiplier(prevBlock, nextFeeMultiplier);

          const valid = isChangeDirectionValid(blockRecord, change);

          debug(`Block #${blockNum} - Fee multiplier change: ${change}, Valid: ${valid}`);
          debug(
            `  Previous multiplier: ${prevBlock ? prevBlock.nextFeeMultiplier.toString() : "N/A"}, Current multiplier: ${nextFeeMultiplier.toString()}`
          );

          // Calculate block weight as percentage of target weight
          const blockWeightPercentage = (
            (Number(blockWeightValue) / Number(targetWeight)) *
            100
          ).toFixed(2);
          debug(
            `  Block weight: ${blockWeightValue} (${blockWeightPercentage}% of target ${targetWeight})`
          );

          return {
            blockNum,
            change,
            shouldIncrease: blockWeightValue >= targetWeight,
            limitingDimension,
            normalizedRefTime,
            normalizedProofSize,
            blockWeightValue,
            targetWeight,
            valid,
          };
        });

        const failures = enriched.filter(({ valid }) => !valid);
        failures.forEach(
          ({
            blockNum,
            change,
            shouldIncrease,
            limitingDimension,
            normalizedRefTime,
            normalizedProofSize,
            blockWeightValue,
            targetWeight,
          }) => {
            // Extract values directly from Perbill objects for display
            const refTimeRatio = normalizedRefTime.toString();
            const proofSizeRatio = normalizedProofSize.toString();

            // Format values for display only
            const refTimePercentFormatted = (Number(refTimeRatio) / 10_000_000).toFixed(2);
            const proofSizePercentFormatted = (Number(proofSizeRatio) / 10_000_000).toFixed(2);

            // Get the block record to access multiplier values
            const blockRecord = blockData.find((b) => b.blockNum === blockNum);
            const { minMultiplier, maxMultiplier, nextFeeMultiplier } = blockRecord ?? {};
            const feeMultiplier = nextFeeMultiplier?.toBigInt();

            debug(
              `Block #${blockNum} with feeMultiplier ${change}\n` +
                `  Limiting dimension: ${limitingDimension}\n` +
                `  Normalized ref_time: ${refTimePercentFormatted}%, proof_size: ${proofSizePercentFormatted}%\n` +
                `  Target weight: ${targetWeight}, Block weight: ${blockWeightValue}\n` +
                `  Should increase multiplier: ${shouldIncrease}\n` +
                `  Current multiplier: ${feeMultiplier}, MIN: ${minMultiplier}, MAX: ${maxMultiplier}`
            );
          }
        );

        expect(
          failures.length,
          `Please investigate blocks ${failures.map(({ blockNum }) => blockNum).join(`, `)}`
        ).to.equals(0);
      },
    });

    it({
      id: "C200",
      title: "Block utilization from normal class exts corresponds to fee multiplier",
      timeout: 30000,
      test: async function () {
        if (skipAll) {
          log("Skipping test suite due to runtime version");
          return;
        }
        const enriched = blockData.map((blockRecord) => {
          const {
            blockNum,
            limitingDimension,
            normalizedRefTime,
            normalizedProofSize,
            blockWeightValue,
            targetWeight,
          } = blockRecord;

          const prevBlock = allBlocks.find((blockDatum) => blockDatum.blockNum === blockNum - 1)!;
          const change = checkMultiplier(prevBlock, blockRecord.nextFeeMultiplier);

          const valid = isChangeDirectionValid(blockRecord, change);

          debug(`Block #${blockNum} - Fee multiplier change: ${change}, Valid: ${valid}`);
          debug(
            `  Previous multiplier: ${prevBlock ? prevBlock.nextFeeMultiplier.toString() : "N/A"}, Current multiplier: ${blockRecord.nextFeeMultiplier.toString()}`
          );

          return {
            blockNum,
            change,
            valid,
            limitingDimension,
            normalizedRefTime,
            normalizedProofSize,
            blockWeightValue,
            targetWeight,
            shouldIncrease: blockWeightValue >= targetWeight,
          };
        });

        const failures = enriched.filter(({ valid }) => !valid);
        failures.forEach(
          ({
            blockNum,
            change,
            limitingDimension,
            normalizedRefTime,
            normalizedProofSize,
            blockWeightValue,
            targetWeight,
            shouldIncrease,
          }) => {
            // Extract values directly from Perbill objects for display
            const refTimeRatio = normalizedRefTime.toString();
            const proofSizeRatio = normalizedProofSize.toString();

            // Format values for display only
            const refTimePercentFormatted = (Number(refTimeRatio) / 10_000_000).toFixed(2);
            const proofSizePercentFormatted = (Number(proofSizeRatio) / 10_000_000).toFixed(2);

            // Get the block record to access multiplier values
            const blockRecord = blockData.find((b) => b.blockNum === blockNum);
            const { minMultiplier, maxMultiplier, nextFeeMultiplier } = blockRecord ?? {};
            const feeMultiplier = nextFeeMultiplier?.toBigInt();

            debug(
              `Block #${blockNum} with feeMultiplier ${change}\n` +
                `  Limiting dimension: ${limitingDimension}\n` +
                `  Normalized ref_time: ${refTimePercentFormatted}%, proof_size: ${proofSizePercentFormatted}%\n` +
                `  Target weight: ${targetWeight}, Block weight: ${blockWeightValue}\n` +
                `  Should increase multiplier: ${shouldIncrease}\n` +
                `  Current multiplier: ${feeMultiplier}, MIN: ${minMultiplier}, MAX: ${maxMultiplier}`
            );
          }
        );
        expect(
          failures.length,
          `Please investigate blocks ${failures.map(({ blockNum }) => blockNum).join(`, `)}`
        ).to.equals(0);
      },
    });

    it({
      id: "C300",
      title: "BaseFeePerGas is within expected min/max",
      timeout: 30000,
      test: function () {
        if (skipAll) {
          log("Skipping test suite due to runtime version");
          return;
        }
        const failures = blockData.filter(({ baseFeePerGasInGwei }) => {
          return (
            ethers.parseUnits(baseFeePerGasInGwei, "gwei") <
              RUNTIME_CONSTANTS[runtime].MIN_BASE_FEE.get(specVersion.toNumber()) ||
            ethers.parseUnits(baseFeePerGasInGwei, "gwei") >
              RUNTIME_CONSTANTS[runtime].MAX_BASE_FEE.get(specVersion.toNumber())
          );
        });

        failures.forEach(({ blockNum, baseFeePerGasInGwei }) => {
          log(`Block #${blockNum} has baseFeePerGas out of range: ${baseFeePerGasInGwei}`);
        });
        expect(
          failures.length,
          `Please investigate blocks ${failures.map(({ blockNum }) => blockNum).join(`, `)}`
        ).to.equals(0);
      },
    });

    it({
      id: "C400",
      title: "BaseFeePerGas is correctly calculated",
      timeout: 30000,
      test: async () => {
        if (skipAll) {
          log("Skipping test suite due to runtime version");
          return;
        }
        const runtime = paraApi.consts.system.version.specName.toUpperCase();
        const weightFee = RUNTIME_CONSTANTS[runtime].WEIGHT_FEE.get(specVersion.toNumber());

        const failures = blockData
          .map(({ blockNum, nextFeeMultiplier, baseFeePerGasInGwei }) => {
            const baseFeePerGasInWei = ethers.parseUnits(baseFeePerGasInGwei, "gwei");

            const expectedBaseFeePerGasInWei =
              (nextFeeMultiplier.toBigInt() * weightFee * WEIGHT_PER_GAS) / ethers.parseEther("1");

            const valid = baseFeePerGasInWei === expectedBaseFeePerGasInWei;
            return { blockNum, baseFeePerGasInGwei, valid, expectedBaseFeePerGasInWei };
          })
          .filter(({ valid }) => !valid);

        failures.forEach(({ blockNum, baseFeePerGasInGwei, expectedBaseFeePerGasInWei }) => {
          const expected = `expected: ${ethers.formatUnits(expectedBaseFeePerGasInWei, "gwei")}`;
          log(
            `Block #${blockNum} has incorrect baseFeePerGas: ${baseFeePerGasInGwei}, ${expected}`
          );
        });
        expect(
          failures.length,
          `Please investigate blocks ${failures.map(({ blockNum }) => blockNum).join(`, `)}`
        ).to.equals(0);
      },
    });

    // This test will: 1) get the gasUsed metric contained in transaction receipts for corresponding
    // ethereum.transact extrinsics 2) calculate the corresponding fee from gasUsed 3) Compare that
    // against the actual weight fee charged. If this test seems overly complicated, it is because
    // we wish to only compare evm transactions, not those generated by XCM calls.

    it({
      id: "C500",
      title: "Ext fee matches on chain",
      timeout: 30000,
      test: function () {
        if (skipAll) {
          log("Skipping test suite due to runtime version");
          return;
        }

        const extractGasAmount = (item: EthereumReceiptReceiptV3) => {
          switch (true) {
            case item.isEip1559:
              return item.asEip1559.usedGas.toBigInt();
            case item.isEip2930:
              return item.asEip2930.usedGas.toBigInt();
            case item.isLegacy:
              return item.asLegacy.usedGas.toBigInt();
            default:
              throw new Error("Update test to include new transaction type");
          }
        };

        const isEthereumTxn = (blockNum: number, index: number) => {
          const extrinsic = blockData.find((blockDatum) => blockDatum.blockNum === blockNum)!
            .extrinsics[index];
          return (
            extrinsic.method.section.toString() === "ethereum" &&
            extrinsic.method.method.toString() === "transact"
          );
        };

        const filteredEvents = blockData
          .map(({ blockNum, events, receipts, transactionStatuses }) => {
            const matchedEvents = events.filter((emittedEvent) =>
              paraApi.events.system.ExtrinsicSuccess.is(emittedEvent.event)
            );

            const filteredTxnEvents = events.filter((emittedEvent) => {
              return paraApi.events.ethereum.Executed.is(emittedEvent.event);
            });

            return { blockNum, matchedEvents, filteredTxnEvents, receipts, transactionStatuses };
          })
          .filter(({ matchedEvents }) => matchedEvents.length > 0);

        const failures = filteredEvents
          .map(({ blockNum, matchedEvents, filteredTxnEvents, transactionStatuses, receipts }) => {
            const fees = matchedEvents
              .filter((emittedEvent) =>
                isEthereumTxn(blockNum, emittedEvent.phase.asApplyExtrinsic.toNumber())
              )
              .map(({ event }) => {
                const info = event.data[0] as DispatchInfo;
                const fee = info.weight as SpWeightsWeightV2Weight;
                return fee.refTime;
              });

            const gasUsed = filteredTxnEvents.flatMap((txnEvent) => {
              if (
                txnEvent.phase.isApplyExtrinsic && // Exclude XCM => EVM calls
                isEthereumTxn(blockNum, txnEvent.phase.asApplyExtrinsic.toNumber())
              ) {
                const txnHash = (txnEvent.event.data as any).transactionHash;
                const index = transactionStatuses.findIndex((status) =>
                  status.transactionHash.eq(txnHash)
                );
                // Gas used is cumulative measure, so we have to derive individuals
                const gasUsed =
                  index === 0
                    ? extractGasAmount(receipts[index])
                    : extractGasAmount(receipts[index]) - extractGasAmount(receipts[index - 1]);
                return gasUsed;
              }
              return [];
            });

            expect(fees.length, "More eth reciepts than expected, this test needs fixing").to.equal(
              gasUsed.length
            );

            const estimatedFees = gasUsed.map((amount) => amount * WEIGHT_PER_GAS);

            const matchedAmounts = estimatedFees
              .map((a, index) => a === fees[index].toBigInt().valueOf())
              .reduce((curr, arr) => curr && arr, true);

            return { blockNum, fees, gasUsed, estimatedFees, matchedAmounts };
          })
          .filter((item) => item.matchedAmounts !== true);

        failures.forEach((blockDatum) => {
          log(
            `Block #${blockDatum.blockNum}:\n\tgasUsed: [${blockDatum.gasUsed.map((amt) =>
              amt.toString()
            )}]\n\tfees: [${blockDatum.fees.map((amt) =>
              amt.toString()
            )}]\n\testimatedFees: [${blockDatum.estimatedFees.map((amt) => amt.toString())}]`
          );
        });

        expect(
          failures.length,
          `Please investigate blocks ${failures.map(({ blockNum }) => blockNum).join(`, `)}`
        ).to.equal(0);
      },
    });
  },
});

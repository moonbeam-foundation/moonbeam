import "@moonbeam-network/api-augment/moonbase";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import {
  RUNTIME_CONSTANTS,
  TARGET_FILL_PERMILL,
  WEIGHT_FEE,
  WEIGHT_PER_GAS,
  getBlockArray,
} from "@moonwall/util";
import { ApiPromise } from "@polkadot/api";
import { GenericExtrinsic, u256 } from "@polkadot/types";
import type { u128 } from "@polkadot/types-codec";
import type { DispatchInfo } from "@polkadot/types/interfaces";
import {
  EthereumBlock,
  EthereumReceiptReceiptV3,
  FpRpcTransactionStatus,
  FrameSupportDispatchPerDispatchClassWeight,
  FrameSystemEventRecord,
  SpWeightsWeightV2Weight,
} from "@polkadot/types/lookup";
import { AnyTuple } from "@polkadot/types/types";
import { BN, BN_MILLION } from "@polkadot/util";
import { ethers } from "ethers";
import { checkTimeSliceForUpgrades, rateLimiter } from "../../helpers/common.js";
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
};

enum Change {
  Increased = "INCREASED",
  Decreased = "DECREASED",
  Unchanged = "UNCHANGED",
  Unknown = "UNKNOWN",
  Invalid = "INVALID",
}

describeSuite({
  id: "S550",
  title: `Dynamic fees in past ${hours} hours should be correct`,
  foundationMethods: "read_only",
  testCases: ({ context, it, log }) => {
    let blockData: BlockFilteredRecord[];
    let runtime: "MOONRIVER" | "MOONBEAM" | "MOONBASE";
    let paraApi: ApiPromise;
    let skipAll = false;

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

    const isChangeDirectionValid = (fillPermill: BN, change: Change, feeMultiplier: BN) => {
      switch (true) {
        case fillPermill.gt(new BN(TARGET_FILL_PERMILL)) && change == Change.Increased:
          return true;
        case fillPermill.gt(new BN(TARGET_FILL_PERMILL)) &&
          change == Change.Unchanged &&
          feeMultiplier.eq(new BN(RUNTIME_CONSTANTS[runtime].MAX_FEE_MULTIPLIER)):
          return true;
        case fillPermill.lt(new BN(TARGET_FILL_PERMILL)) && change == Change.Decreased:
          return true;
        case fillPermill.lt(new BN(TARGET_FILL_PERMILL)) &&
          change == Change.Unchanged &&
          feeMultiplier.eq(new BN(RUNTIME_CONSTANTS[runtime].MIN_FEE_MULTIPLIER)):
          return true;
        case fillPermill.eq(new BN(TARGET_FILL_PERMILL)) && change == Change.Unchanged:
          return true;
        case change == Change.Unknown:
          return true;
        default:
          return false;
      }
    };

    beforeAll(async function () {
      paraApi = context.polkadotJs("para");
      const { specVersion, specName } = paraApi.consts.system.version;
      runtime = specName.toUpperCase() as any;

      if (
        specVersion.toNumber() < 2200 &&
        (specName.toString() == "moonbase" || specName.toString() == "moonriver")
      ) {
        log(
          `Runtime ${specName.toString()} version ` +
            `${specVersion.toString()} is less than 2200, skipping test suite.`
        );
        skipAll = true;
      }

      if (specVersion.toNumber() < 2300 && specName.toString() == "moonbeam") {
        log(
          `Runtime ${specName.toString()} version ` +
            `${specVersion.toString()} is less than 2300, skipping test suite.`
        );
        skipAll = true;
      }

      const blockNumArray = atBlock > 0 ? [atBlock] : await getBlockArray(paraApi, timePeriod);

      log(`Collecting ${hours} hours worth of block data`);

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
        return {
          blockNum: blockNum,
          nextFeeMultiplier: await apiAt.query.transactionPayment.nextFeeMultiplier(),
          ethBlock,
          ethersTransactionsFees,
          transactionStatuses,
          extrinsics: signedBlock.block.extrinsics,
          baseFeePerGasInGwei: ethers.formatUnits(ethersBlock!.baseFeePerGas!, "gwei"),
          weights,
          receipts,
          events,
        };
      };

      // Determine if the block range intersects with an upgrade event
      const { result, specVersion: onChainRt } = await checkTimeSliceForUpgrades(
        paraApi,
        blockNumArray,
        specVersion
      );
      if (result) {
        log(`Time slice of blocks intersects with upgrade from RT ${onChainRt}, skipping tests.`);
        skipAll = true;
      }

      blockData = await Promise.all(
        blockNumArray.map((num) => limiter.schedule(() => getBlockData(num)))
      );
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
        const maxWeights = paraApi.consts.system.blockWeights;
        const enriched = blockData.map(({ weights, blockNum, nextFeeMultiplier }) => {
          const fillPermill = weights.normal.refTime
            .toBn()
            .mul(new BN("1000000"))
            .div(maxWeights.perClass.normal.maxTotal.unwrap().refTime.toBn());

          const change = checkMultiplier(
            blockData.find((blockDatum) => blockDatum.blockNum == blockNum - 1)!,
            nextFeeMultiplier
          );

          return {
            blockNum,
            fillPermill,
            change,
            valid: isChangeDirectionValid(fillPermill, change, nextFeeMultiplier),
          };
        });

        const failures = enriched.filter(({ valid }) => !valid);
        failures.forEach(({ blockNum, fillPermill, change }) => {
          log(
            `Block #${blockNum} is ${(fillPermill.toNumber() / 1_000_000).toFixed(
              2
            )}% full with feeMultiplier ${change}`
          );
        });

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
        const enriched = blockData.map(({ blockNum, nextFeeMultiplier, weights }) => {
          const fillPermill = weights.normal.refTime
            .unwrap()
            .toBn()
            .mul(BN_MILLION)
            .div(
              paraApi.consts.system.blockWeights.perClass.normal.maxTotal.unwrap().refTime.toBn()
            );

          const change = checkMultiplier(
            blockData.find((blockDatum) => blockDatum.blockNum == blockNum - 1)!,
            nextFeeMultiplier
          );
          const valid = isChangeDirectionValid(
            new BN(fillPermill.toString()),
            change,
            nextFeeMultiplier
          );

          return { blockNum, fillPermill, change, valid };
        });

        const failures = enriched.filter(({ valid }) => !valid);
        failures.forEach(({ blockNum, fillPermill, change }) => {
          log(
            `Block #${blockNum} is ${(fillPermill.toNumber() / 1_000_000).toFixed(
              2
            )}% full with feeMultiplier ${change}`
          );
        });
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
              BigInt(RUNTIME_CONSTANTS[runtime].MIN_BASE_FEE_IN_WEI) ||
            ethers.parseUnits(baseFeePerGasInGwei, "gwei") >
              BigInt(RUNTIME_CONSTANTS[runtime].MAX_BASE_FEE_IN_WEI)
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
      test: function () {
        if (skipAll) {
          log("Skipping test suite due to runtime version");
          return;
        }
        const supplyFactor =
          paraApi.consts.system.version.specName.toString() === "moonbeam" ? 100n : 1n;

        const failures = blockData
          .map(({ blockNum, nextFeeMultiplier, baseFeePerGasInGwei }) => {
            const baseFeePerGasInWei = ethers.parseUnits(baseFeePerGasInGwei, "gwei");

            const expectedBaseFeePerGasInWei =
              (nextFeeMultiplier.toBigInt() * WEIGHT_FEE * WEIGHT_PER_GAS * supplyFactor) /
              ethers.parseEther("1");

            const valid = baseFeePerGasInWei == expectedBaseFeePerGasInWei;
            return { blockNum, baseFeePerGasInGwei, valid };
          })
          .filter(({ valid }) => !valid);

        failures.forEach(({ blockNum, baseFeePerGasInGwei }) => {
          log(`Block #${blockNum} has incorrect baseFeePerGas: ${baseFeePerGasInGwei}`);
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
              return item.asEip1559.usedGas;
            case item.isEip2930:
              return item.asEip2930.usedGas;
            case item.isLegacy:
              return item.asLegacy.usedGas;
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

            const gasUsed: u256[] = filteredTxnEvents
              .map((txnEvent) => {
                if (isEthereumTxn(blockNum, txnEvent.phase.asApplyExtrinsic.toNumber())) {
                  const txnHash = (txnEvent.event.data as any).transactionHash;
                  const index = transactionStatuses.findIndex((status) =>
                    status.transactionHash.eq(txnHash)
                  );
                  // Gas used is cumulative measure, so we have to derive individuals
                  const gasUsed =
                    index === 0
                      ? extractGasAmount(receipts[index])
                      : extractGasAmount(receipts[index]).sub(
                          extractGasAmount(receipts[index - 1])
                        );
                  return gasUsed;
                } else {
                  return [];
                }
              })
              .flatMap((item) => item);

            expect(fees.length, "More eth reciepts than expected, this test needs fixing").to.equal(
              gasUsed.length
            );

            const estimatedFees = gasUsed.map((amount) =>
              amount.mul(new BN(WEIGHT_PER_GAS.toString()))
            );

            const matchedAmounts = estimatedFees
              .map((a, index) => a.eq(fees[index].toBn()).valueOf())
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

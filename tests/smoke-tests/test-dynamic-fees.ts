import "@moonbeam-network/api-augment/moonbase";
import { expect } from "chai";
import { getBlockArray } from "../util/block";
import { describeSmokeSuite } from "../util/setup-smoke-tests";
import type { DispatchInfo } from "@polkadot/types/interfaces";
import Bottleneck from "bottleneck";
import { BigNumber, ethers } from "ethers";
import { GenericExtrinsic } from "@polkadot/types";
import {
  FrameSupportDispatchPerDispatchClassWeight,
  EthereumBlock,
  FrameSystemEventRecord,
  EthereumReceiptReceiptV3,
  SpWeightsWeightV2Weight,
} from "@polkadot/types/lookup";
import { AnyTuple } from "@polkadot/types/types";
import BN from "bn.js";
import type { u128 } from "@polkadot/types-codec";
import {
  RUNTIME_CONSTANTS,
  TARGET_FILL_PERMILL,
  WEIGHT_FEE,
  WEIGHT_PER_GAS,
} from "../util/constants";
import { BN_MILLION } from "@polkadot/util";
const debug = require("debug")("smoke:dynamic-fees");
const timePeriod = process.env.TIME_PERIOD ? Number(process.env.TIME_PERIOD) : 2 * 60 * 60 * 1000;
const timeout = Math.max(Math.floor(timePeriod / 12), 5000);
const limiter = new Bottleneck({ maxConcurrent: 10, minTime: 100 });
const hours = (timePeriod / (1000 * 60 * 60)).toFixed(2);

type BlockFilteredRecord = {
  blockNum: number;
  nextFeeMultiplier: u128;
  ethBlock: EthereumBlock;
  extrinsics: GenericExtrinsic<AnyTuple>[];
  ethersTransactionsFees: BigNumber[];
  baseFeePerGasInGwei: string;
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

describeSmokeSuite(
  "S550",
  `Dynamic fees in past ${hours} hours should be correct`,

  (context, testIt) => {
    let blockData: BlockFilteredRecord[];
    let runtime: "MOONRIVER" | "MOONBEAM" | "MOONBASE";

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

    before("Retrieve events for previous blocks", async function () {
      this.timeout(timeout);
      const { specVersion, specName } = context.polkadotApi.consts.system.version;
      runtime = specName.toUpperCase() as any;

      if (specVersion.toNumber() < 2200 && specName.toString() == "moonbase") {
        debug(`Runtime version ${specVersion.toString()} is less than 2200, skipping test suite.`);
        this.skip();
      }

      if (specVersion.toNumber() < 2200 && specName.toString() == "moonriver") {
        debug(
          `Runtime version ${specVersion.toString()} for ${specName.toString()}` +
            ` is less than 2100, skipping test suite.`
        );
        this.skip();
      }

      if (specName.toString() == "moonbeam") {
        debug(`Runtime ${specName.toString()} not supported by these tests, skipping.`);
        this.skip();
      }

      const blockNumArray = await getBlockArray(context.polkadotApi, timePeriod, limiter);

      debug(`Collecting ${hours} hours worth of block data`);

      const getBlockData = async (blockNum: number) => {
        const blockHash = await context.polkadotApi.rpc.chain.getBlockHash(blockNum);
        const signedBlock = await context.polkadotApi.rpc.chain.getBlock(blockHash);
        const apiAt = await context.polkadotApi.at(blockHash);
        const ethBlock = (await apiAt.query.ethereum.currentBlock()).unwrapOrDefault();
        const ethersBlock = await context.ethers.getBlock(blockNum);
        const ethersTransactionsFees = await Promise.all(
          ethersBlock.transactions.map(
            async (a) => (await context.ethers.getTransactionReceipt(a)).gasUsed
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
          extrinsics: signedBlock.block.extrinsics,
          baseFeePerGasInGwei: ethers.utils.formatUnits(ethersBlock.baseFeePerGas, "gwei"),
          weights,
          receipts,
          events,
        };
      };

      blockData = await Promise.all(
        blockNumArray.map((num) => limiter.schedule(() => getBlockData(num)))
      );
    });

    testIt("C100", "Block utilization by weight corresponds to fee multiplier", function () {
      this.timeout(30000);
      const maxWeights = context.polkadotApi.consts.system.blockWeights;
      const enriched = blockData.map(({ weights, blockNum, nextFeeMultiplier }) => {
        const fillPermill = weights.normal.refTime
          .toBn()
          .mul(new BN("1000000"))
          .div(maxWeights.perClass.normal.maxTotal.unwrap().refTime.toBn());

        const change = checkMultiplier(
          blockData.find((a) => a.blockNum == blockNum - 1),
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
        debug(
          `Block #${blockNum} is ${(fillPermill.toNumber() / 1_000_000).toFixed(
            2
          )}% full with feeMultiplier ${change}`
        );
      });

      expect(
        failures.length,
        `Please investigate blocks ${failures.map(({ blockNum }) => blockNum).join(`, `)}`
      ).to.equals(0);
    });

    testIt(
      "C200",
      "Block utilization from normal class exts corresponds to fee multiplier",
      async function () {
        this.timeout(30000);
        const enriched = blockData.map(({ blockNum, nextFeeMultiplier, weights }) => {
          const fillPermill = weights.normal.refTime
            .unwrap()
            .toBn()
            .mul(BN_MILLION)
            .div(
              context.polkadotApi.consts.system.blockWeights.perClass.normal.maxTotal
                .unwrap()
                .refTime.toBn()
            );

          const change = checkMultiplier(
            blockData.find((a) => a.blockNum == blockNum - 1),
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
          debug(
            `Block #${blockNum} is ${(fillPermill.toNumber() / 1_000_000).toFixed(
              2
            )}% full with feeMultiplier ${change}`
          );
        });
        expect(
          failures.length,
          `Please investigate blocks ${failures.map(({ blockNum }) => blockNum).join(`, `)}`
        ).to.equals(0);
      }
    );

    testIt("C300", "BaseFeePerGas is within expected min/max", function () {
      this.timeout(30000);
      const failures = blockData.filter(({ baseFeePerGasInGwei }) => {
        return (
          ethers.utils
            .parseUnits(baseFeePerGasInGwei, "gwei")
            .lt(BigNumber.from(RUNTIME_CONSTANTS[runtime].MIN_BASE_FEE_IN_WEI)) ||
          ethers.utils
            .parseUnits(baseFeePerGasInGwei, "gwei")
            .gt(BigNumber.from(RUNTIME_CONSTANTS[runtime].MAX_BASE_FEE_IN_WEI))
        );
      });

      failures.forEach(({ blockNum, baseFeePerGasInGwei }) => {
        debug(`Block #${blockNum} has baseFeePerGas out of range: ${baseFeePerGasInGwei}`);
      });
      expect(
        failures.length,
        `Please investigate blocks ${failures.map(({ blockNum }) => blockNum).join(`, `)}`
      ).to.equals(0);
    });

    testIt("C400", "BaseFeePerGas is correctly calculated", function () {
      this.timeout(30000);
      const supplyFactor =
        context.polkadotApi.consts.system.version.specName.toString() === "moonbeam" ? 100n : 1n;

      const failures = blockData
        .map(({ blockNum, nextFeeMultiplier, baseFeePerGasInGwei }) => {
          const baseFeePerGasInWei = ethers.utils.parseUnits(baseFeePerGasInGwei, "gwei");

          const expectedBaseFeePerGasInWei =
            (nextFeeMultiplier.toBigInt() * WEIGHT_FEE * WEIGHT_PER_GAS * supplyFactor) /
            ethers.utils.parseEther("1").toBigInt();

          const valid = baseFeePerGasInWei.eq(BigNumber.from(expectedBaseFeePerGasInWei));
          return { blockNum, baseFeePerGasInGwei, valid };
        })
        .filter(({ valid }) => !valid);

      failures.forEach(({ blockNum, baseFeePerGasInGwei }) => {
        debug(`Block #${blockNum} has incorrect baseFeePerGas: ${baseFeePerGasInGwei}`);
      });
      expect(
        failures.length,
        `Please investigate blocks ${failures.map(({ blockNum }) => blockNum).join(`, `)}`
      ).to.equals(0);
    });

    testIt("C500", "Ext fee matches on chain", function () {
      this.timeout(30000);
      const filteredEvents = blockData
        .map(({ blockNum, events, receipts, ethersTransactionsFees }) => {
          const matchedEvents = events.filter(({ event }) =>
            context.polkadotApi.events.system.ExtrinsicSuccess.is(event)
          );
          return { blockNum, matchedEvents, receipts, ethersTransactionsFees };
        })
        .filter(({ matchedEvents }) => matchedEvents.length > 0);

      const isEthereumTxn = (blockNum: number, index: number) => {
        const extrinsic = blockData.find((a) => a.blockNum === blockNum).extrinsics[index];
        return (
          extrinsic.method.section.toString() === "ethereum" &&
          extrinsic.method.method.toString() === "transact"
        );
      };

      const failures = filteredEvents
        .map(({ blockNum, matchedEvents, receipts, ethersTransactionsFees }) => {
          const ethExtFees = matchedEvents
            .filter((a) => isEthereumTxn(blockNum, a.phase.asApplyExtrinsic.toNumber()))
            .map(({ event }, index) => {
              const info = event.data[0] as DispatchInfo;
              const fee = info.weight as unknown as SpWeightsWeightV2Weight;
              return fee;
            });
          const ethExtGas = receipts.map((item) => {
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
          });
          return { blockNum, ethExtFees, ethExtGas, ethersTransactionsFees };
        })
        .filter((a) => a.ethExtFees.length > 0)
        .filter((item) => {
          const match = item.ethExtFees
            .map((subitem, index) =>
              subitem.refTime.eq(item.ethersTransactionsFees[index].mul(WEIGHT_PER_GAS.toString()))
            )
            .reduce((acc, curr) => acc && curr, true);

          return !match;
        });

      failures.forEach(({ blockNum, ethExtFees, ethExtGas }) => {
        debug(
          `Incorrect Block #${blockNum}, fees: ` +
            `[${ethExtFees.map((a) => a.refTime).toString()}] vs gas: [${ethExtGas.toString()}]`
        );
      });

      expect(
        failures.length,
        `Please investigate blocks ${failures.map(({ blockNum }) => blockNum).join(`, `)}`
      ).to.equal(0);
    });
  }
);

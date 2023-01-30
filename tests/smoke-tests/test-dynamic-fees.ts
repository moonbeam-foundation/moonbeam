import "@moonbeam-network/api-augment/moonbase";
import { expect } from "chai";
import { getBlockArray } from "../util/block";
import { describeSmokeSuite } from "../util/setup-smoke-tests";
import Bottleneck from "bottleneck";
import { BigNumber, ethers } from "ethers";
import { FrameSupportDispatchPerDispatchClassWeight, EthereumBlock } from "@polkadot/types/lookup";
import BN from "bn.js";
import type { u128 } from "@polkadot/types-codec";
import {
  RUNTIME_CONSTANTS,
  TARGET_FILL_PERMILL,
  WEIGHT_FEE,
  WEIGHT_PER_GAS,
} from "../util/constants";
const debug = require("debug")("smoke:dynamic-fees");
const timePeriod = process.env.TIME_PERIOD ? Number(process.env.TIME_PERIOD) : 2 * 60 * 60 * 1000;
const timeout = Math.max(Math.floor(timePeriod / 12), 5000);
const limiter = new Bottleneck({ maxConcurrent: 10, minTime: 100 });
const hours = (timePeriod / (1000 * 60 * 60)).toFixed(2);

type BlockFilteredRecord = {
  blockNum: number;
  nextFeeMultiplier: u128;
  ethBlock: EthereumBlock;
  ethersBlock: ethers.providers.Block;
  baseFeePerGasInGwei: string;
  weights: FrameSupportDispatchPerDispatchClassWeight;
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

      runtime = context.polkadotApi.consts.system.version.specName.toUpperCase() as any;

      const { specVersion, specName } = context.polkadotApi.consts.system.version;
      if (specVersion.toNumber() < 2100) {
        debug(`Runtime version ${specVersion.toString()} is less than 2100, skipping test suite.`);
        this.skip();
      }

      if (specName.toString() !== "moonbase") {
        debug(`Runtime ${specName.toString()} not supported by these tests, skipping.`);
        this.skip();
      }

      const blockNumArray = await getBlockArray(context.polkadotApi, timePeriod, limiter);

      debug(`Collecting ${hours} hours worth of block data`);

      const getBlockData = async (blockNum: number) => {
        const blockHash = await context.polkadotApi.rpc.chain.getBlockHash(blockNum);
        const apiAt = await context.polkadotApi.at(blockHash);
        const ethBlock = (await apiAt.query.ethereum.currentBlock()).unwrapOrDefault();
        const ethersBlock = await context.ethers.getBlock(blockNum);
        const weights = await apiAt.query.system.blockWeight();
        return {
          blockNum: blockNum,
          nextFeeMultiplier: await apiAt.query.transactionPayment.nextFeeMultiplier(),
          ethBlock,
          ethersBlock,
          baseFeePerGasInGwei: ethers.utils.formatUnits(ethersBlock.baseFeePerGas, "gwei"),
          weights,
        };
      };

      blockData = await Promise.all(
        blockNumArray.map((num) => limiter.schedule(() => getBlockData(num)))
      );
    });

    testIt("C100", "Block utilization by weight corresponds to fee multiplier", function () {
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

    testIt("C200", "Block utilization by gas corresponds to fee multiplier", async function () {
      const enriched = blockData.map(({ blockNum, ethersBlock, nextFeeMultiplier }) => {
        const fillPermill = ethersBlock.gasUsed.mul("1000000").div(ethersBlock.gasLimit);
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
    });

    testIt("C300", "baseFeePerGas is within expected min/max", function () {
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

    testIt("C500", "BaseFeePerGas reported in emulated block header", function () {
      this.skip(); // Feature-under-test not implemented yet

      /*  -- UNCOMMENT AND WIRE UP WHEN FEATURE DELIVERED --
      const failures = blockData
          .map(({ blockNum, ethBlock, baseFeePerGasInGwei }) => {
            const baseFeePerGasInWei = ethers.utils.parseUnits(baseFeePerGasInGwei, "gwei").toString()
            const valid = ethBlock.header.BaseFee.toBn().eq( new BN(baseFeePerGasInWei))
            return { blockNum, valid };
          })
          .filter(({ valid }) => !valid);

          failures.forEach(({ blockNum }) => {
            debug(`Block #${blockNum} reporting incorrect BaseFee in header`);
          });
          expect(
            failures.length,
            `Please investigate blocks ${failures.map(({ blockNum }) => blockNum).join(`, `)}`
          ).to.equals(0);
          */
    });
  }
);

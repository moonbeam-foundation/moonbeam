import "@moonbeam-network/api-augment/moonbase";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { getBlockArray } from "@moonwall/util";
import { ApiPromise } from "@polkadot/api";
import { GenericExtrinsic } from "@polkadot/types";
import type { u128 } from "@polkadot/types-codec";
import {
  EthereumBlock,
  EthereumReceiptReceiptV3,
  FpRpcTransactionStatus,
  FrameSupportDispatchPerDispatchClassWeight,
  FrameSystemEventRecord,
} from "@polkadot/types/lookup";
import { AnyTuple } from "@polkadot/types/types";
import { ethers } from "ethers";
import { checkTimeSliceForUpgrades, rateLimiter, RUNTIME_CONSTANTS } from "../../helpers";
import Debug from "debug";
const debug = Debug("smoke:dynamic-fees");

const timePeriod = process.env.TIME_PERIOD ? Number(process.env.TIME_PERIOD) : 2 * 60 * 60 * 1000;
const timeout = Math.floor(timePeriod / 12); // 2 hour -> 10 minute timeout
const limiter = rateLimiter();
const hours = (timePeriod / (1000 * 60 * 60)).toFixed(2);
const atBlock = process.env.AT_BLOCK ? Number(process.env.AT_BLOCK) : -1;

type BlockFilteredRecord = {
  blockNum: number;
  nextFeeMultiplier: u128;
  fillPermill: bigint;
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
  id: "S06",
  title: `Dynamic fees in past ${hours} hours should be correct`,
  foundationMethods: "read_only",
  testCases: ({ context, it, log }) => {
    let blockData: BlockFilteredRecord[];
    let allBlocks: BlockFilteredRecord[]; // includes previous block & blocks from blockData
    let runtime: "MOONRIVER" | "MOONBEAM" | "MOONBASE";
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

    const isChangeDirectionValid = (fillPermill: bigint, change: Change, feeMultiplier: bigint) => {
      switch (true) {
        case fillPermill > targetFillPermill && change == Change.Increased:
          return true;
        case fillPermill > targetFillPermill &&
          change == Change.Unchanged &&
          feeMultiplier === RUNTIME_CONSTANTS[runtime].MAX_FEE_MULTIPLIER:
          return true;
        case fillPermill < targetFillPermill && change == Change.Decreased:
          return true;
        case fillPermill < targetFillPermill &&
          change == Change.Unchanged &&
          feeMultiplier === RUNTIME_CONSTANTS[runtime].MIN_FEE_MULTIPLIER:
          return true;
        case fillPermill === targetFillPermill && change == Change.Unchanged:
          return true;
        case change == Change.Unknown:
          return true;
        default:
          return false;
      }
    };

    beforeAll(async function () {
      paraApi = context.polkadotJs("para");

      const blockNumArray = atBlock > 0 ? [atBlock] : await getBlockArray(paraApi, timePeriod);
      const previousBlockNumber = Math.max(blockNumArray[0] - 1, 0);

      // Retrieves version of the last block to check.
      const endsAtBlockHash = await paraApi.rpc.chain.getBlockHash(
        blockNumArray[blockNumArray.length - 1]
      );
      const { specVersion, specName } = (await paraApi.at(endsAtBlockHash)).consts.system.version;
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

      targetFillPermill = RUNTIME_CONSTANTS[runtime].TARGET_FILL_PERMILL.get(
        specVersion.toNumber()
      );

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
        const fillPermill =
          (weights.normal.refTime.unwrap().toBigInt() * 1_000_000n) /
          apiAt.consts.system.blockWeights.perClass.normal.maxTotal.unwrap().refTime.toBigInt();
        debug(`Block #${blockNum} fullness: ${(Number(fillPermill) / 10_000).toFixed(2)}%`);

        return {
          blockNum,
          nextFeeMultiplier,
          fillPermill,
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
      id: "C200",
      title: "Block utilization from normal class exts corresponds to fee multiplier",
      timeout: 30000,
      test: async function () {
        if (skipAll) {
          log("Skipping test suite due to runtime version");
          return;
        }
        const enriched = blockData.map(({ blockNum, nextFeeMultiplier, fillPermill }) => {
          const change = checkMultiplier(
            allBlocks.find((blockDatum) => blockDatum.blockNum == blockNum - 1)!,
            nextFeeMultiplier
          );
          const valid = isChangeDirectionValid(fillPermill, change, nextFeeMultiplier.toBigInt());
          return { blockNum, fillPermill, change, valid };
        });

        const failures = enriched.filter(({ valid }) => !valid);
        failures.forEach(({ blockNum, fillPermill, change }) => {
          log(
            `Block #${blockNum} is ${(Number(fillPermill) / 10_000).toFixed(
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
  },
});

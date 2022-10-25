import "@moonbeam-network/api-augment";
import { BN } from "@polkadot/util";
import { expect } from "chai";
import { describeSmokeSuite } from "../util/setup-smoke-tests";
import Bottleneck from "bottleneck";
import { fetchHistoricBlockNum, getBlockTime } from "../util/block";
import { WEIGHT_PER_GAS } from "../util/constants";
import { FrameSystemEventRecord } from "@polkadot/types/lookup";

const debug = require("debug")("smoke:weights");

const wssUrl = process.env.WSS_URL || null;
const relayWssUrl = process.env.RELAY_WSS_URL || null;
const timePeriod = process.env.TIME_PERIOD ? Number(process.env.TIME_PERIOD) : 2 * 60 * 60 * 1000;
const limiter = new Bottleneck({ maxConcurrent: 10 });

interface BlockInfo {
  blockNum: number;
  hash: string;
  weights: {
    normal: BN;
    operational: BN;
    mandatory: BN;
  };
  events: FrameSystemEventRecord[];
}

interface BlockLimits {
  normal: BN;
  operational: BN;
}

describeSmokeSuite(
  `Verify weights of blocks in the past ${(timePeriod / (1000 * 60 * 60)).toFixed(2)} hours`,
  { wssUrl, relayWssUrl },
  (context) => {
    let blockLimits: BlockLimits;
    let blockInfoArray: BlockInfo[];

    before("Retrieve all weight limits and usage", async function () {
      this.timeout(240000);

      const signedBlock = await context.polkadotApi.rpc.chain.getBlock(
        await context.polkadotApi.rpc.chain.getFinalizedHead()
      );

      const lastBlockNumber = signedBlock.block.header.number.toNumber();
      const lastBlockTime = getBlockTime(signedBlock);

      const firstBlockTime = lastBlockTime - timePeriod;
      debug(`Searching for the block at: ${new Date(firstBlockTime)}`);
      const firstBlockNumber = (await limiter.wrap(fetchHistoricBlockNum)(
        context.polkadotApi,
        lastBlockNumber,
        firstBlockTime
      )) as number;

      const length = lastBlockNumber - firstBlockNumber;
      const blockNumArray = Array.from({ length }, (_, i) => firstBlockNumber + i);
      const limits = context.polkadotApi.consts.system.blockWeights;

      const getLimits = async (blockNum: number) => {
        const blockHash = await context.polkadotApi.rpc.chain.getBlockHash(blockNum);
        const apiAt = await context.polkadotApi.at(blockHash);
        const specVersion = apiAt.consts.system.version.specVersion.toNumber();
        const events = await apiAt.query.system.events();

        if (specVersion >= 1700) {
          const { normal, operational, mandatory } = await apiAt.query.system.blockWeight();
          return {
            blockNum,
            hash: blockHash.toString(),
            weights: {
              normal,
              operational,
              mandatory,
            },
            events,
          };
        }
      };

      blockLimits = {
        normal: new BN(limits.perClass.normal.maxTotal.toJSON() as number),
        operational: new BN(limits.perClass.operational.maxTotal.toJSON() as number),
      };
      blockInfoArray = await Promise.all(
        blockNumArray.map((num) => limiter.schedule(() => getLimits(num)))
      );
    });

    // This test is more for verifying that the test code is correctly returning good quality data
    // that the rest of the test suite performs verification on
    it("should be returning unique block hashes in array", async () => {
      const hashes = blockInfoArray.map((a) => a.hash);
      const set = new Set(hashes);
      expect(hashes.length, "Duplicate hashes in retrieved data, investigate test").to.be.equal(
        set.size
      );
    });

    // Normal class
    it("normal usage should be less than normal dispatch class limits", async function () {
      const overweight = blockInfoArray
        .filter((a) => a.weights.normal.gt(blockLimits.normal))
        .map((a) => {
          debug(
            `Block #${a.blockNum} has weight ${Number(a.weights.normal)} which is above limit!`
          );
          return a;
        });
      expect(
        overweight,
        `These blocks have normal weights in excess of limit, investigate: ${overweight
          .map((a) => a.blockNum)
          .join(", ")}`
      ).to.be.empty;
    });

    // Operational class
    it("operational usage should be less than dispatch class limits", async function () {
      const overweight = blockInfoArray
        .filter((a) => a.weights.operational.gt(blockLimits.operational))
        .map((a) => {
          debug(
            `Block #${a.blockNum} has weight ${Number(a.weights.operational)} which is above limit!`
          );
          return a;
        });
      expect(
        overweight,
        `These blocks have operational weights in excess of limit, investigate: ${overweight
          .map((a) => a.blockNum)
          .join(", ")}`
      ).to.be.empty;
    });

    // This will test that when Block is 20%+ full, its normal weight is mostly explained
    // by eth signed transactions.
    it("should roughly have a block weight mostly composed of transactions", async function () {
      this.timeout(120000);
      debug(
        `Checking #${blockInfoArray[0].blockNum} - #${
          blockInfoArray[blockInfoArray.length - 1].blockNum
        } block weight proportions.`
      );

      const checkBlockWeight = async (blockInfo: BlockInfo) => {
        const apiAt = await context.polkadotApi.at(blockInfo.hash);

        const normalWeight = Number(blockInfo.weights.normal);
        const maxWeight = blockLimits.normal;
        const ethBlock = (await apiAt.query.ethereum.currentBlock()).unwrap();

        const actualWeightUsed = normalWeight / Number(maxWeight);
        if (actualWeightUsed > 0.2) {
          const gasUsed = ethBlock.header.gasUsed.toBigInt();
          const weightCalc = gasUsed * WEIGHT_PER_GAS;
          const newRatio = (normalWeight - Number(weightCalc)) / Number(maxWeight);
          if (newRatio > 0.2) {
            debug(
              `Block #${blockInfo.blockNum} is ${(actualWeightUsed * 100).toFixed(2)}% full with ${
                ethBlock.transactions.length
              } transactions, non-transaction weight: ${(newRatio * 100).toFixed(2)}%`
            );
          }
          return { blockNum: blockInfo.blockNum, nonTxn: newRatio };
        }
      };

      const results = await Promise.all(
        blockInfoArray.map((blockInfo) => limiter.schedule(() => checkBlockWeight(blockInfo)))
      );
      const nonTxnHeavyBlocks = results.filter((a) => a && a.nonTxn > 0.2);
      expect(
        nonTxnHeavyBlocks,
        `These blocks have non-txn weights >20%, please investigate: ${nonTxnHeavyBlocks
          .map((a) => a.blockNum)
          .join(", ")}`
      ).to.be.empty;
    });

    // This will test that the total normal weight reported is roughly the sum of normal class
    // weight events emitted by signed extrinsics
    it("should have total normal weight matching the signed extrinsics", async function () {
      this.timeout(120000);
      debug(
        `Checking if #${blockInfoArray[0].blockNum} - #${
          blockInfoArray[blockInfoArray.length - 1].blockNum
        } extrinsic weights sum up.`
      );

      const checkWeights = (blockInfo: BlockInfo) => {
        const signedExtTotal = blockInfo.events
          .filter(
            (a) => a.event.method == "ExtrinsicSuccess" || a.event.method == "ExtrinsicFailed"
          )
          .filter((a) => (a.event.data as any).dispatchInfo.class.toString() == "Normal")
          .reduce((acc, curr) => acc + (curr.event.data as any).dispatchInfo.weight.toNumber(), 0);
        const normalWeights = Number(blockInfo.weights.normal);
        const difference = (normalWeights - signedExtTotal) / signedExtTotal;
        if (difference > 0.2) {
          debug(
            `Block #${blockInfo.blockNum} signed extrinsic weight - reported: ${signedExtTotal}, 
            accounted: ${normalWeights} (${difference > 0 ? "+" : "-"}${(difference * 100).toFixed(
              2
            )}%).`
          );
        }
        return { blockNum: blockInfo.blockNum, signedExtTotal, normalWeights, difference };
      };

      const results = blockInfoArray.map((blockInfo) => checkWeights(blockInfo));
      const heavyweights = results.filter((a) => a.difference > 0.2);
      expect(
        heavyweights,
        `These blocks have >20% overweight normal weights, please investigate: ${heavyweights
          .map((a) => a.blockNum)
          .join(", ")}`
      ).to.be.empty;
    });

    // This test will compare the total weight of eth transactions versus the reported gasUsed
    // property of  ethereum.currentBlock()
    it("should have total gas charged similar to eth extrinsics", async function () {
      this.timeout(120000);
      debug(
        `Checking if #${blockInfoArray[0].blockNum} - #${
          blockInfoArray[blockInfoArray.length - 1].blockNum
        } weights match gasUsed`
      );

      const compareGasToWeight = async (blockInfo: BlockInfo) => {
        const apiAt = await context.polkadotApi.at(blockInfo.hash);
        const signedBlock = await context.polkadotApi.rpc.chain.getBlock(blockInfo.hash);
        const gasUsed = (await apiAt.query.ethereum.currentBlock())
          .unwrap()
          .header.gasUsed.toNumber();

        const gasWeight = gasUsed * Number(WEIGHT_PER_GAS);
        const ethTxnsWeight = signedBlock.block.extrinsics
          .map((item, index) => {
            if (item.method.method == "transact" && item.method.section == "ethereum") {
              return blockInfo.events
                .filter(({ phase }) => phase.isApplyExtrinsic && phase.asApplyExtrinsic.eq(index))
                .filter(
                  ({ event }) => event.method == "ExtrinsicSuccess" && event.section == "system"
                )
                .reduce(
                  (acc, curr) => acc + (curr.event.data as any).dispatchInfo.weight.toNumber(),
                  0
                );
            } else {
              return 0;
            }
          })
          .reduce((acc, curr) => acc + curr, 0);
        const difference = ethTxnsWeight - gasWeight;

        if (difference > 0) {
          debug(
            `Block #${blockInfo.blockNum} has a ${((difference / ethTxnsWeight) * 100).toFixed(
              2
            )}% discrepancy between eth gas used and weight charged. `
          );
        }
        return { blockNum: blockInfo.blockNum, gasWeight, ethTxnsWeight, difference };
      };

      const results = await Promise.all(
        blockInfoArray.map((blockInfo) => limiter.schedule(() => compareGasToWeight(blockInfo)))
      );
      const discrepancies = results.filter((a) => a.difference > 0);
      expect(
        discrepancies,
        `These blocks have mismatching gas used vs charged weight, 
        please investigate: ${discrepancies.map((a) => a.blockNum).join(", ")}`
      ).to.be.empty;
    });
  }
);

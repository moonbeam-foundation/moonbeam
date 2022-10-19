import "@moonbeam-network/api-augment";
import { BN } from "@polkadot/util";
import { expect } from "chai";
import { describeSmokeSuite } from "../util/setup-smoke-tests";
import Bottleneck from "bottleneck";
import { fetchHistoricBlockNum, getBlockTime } from "../util/block";
import { WEIGHT_PER_GAS } from "../util/constants";

const debug = require("debug")("smoke:weights");

const wssUrl = process.env.WSS_URL || null;
const relayWssUrl = process.env.RELAY_WSS_URL || null;
const limiter = new Bottleneck({ maxConcurrent: 10 });

describeSmokeSuite(`Verify weights of published blocks`, { wssUrl, relayWssUrl }, (context) => {
  before("Retrieve all weight limits and usage", async function () {
    this.timeout(240000);

    const signedBlock = await context.polkadotApi.rpc.chain.getBlock(
      await context.polkadotApi.rpc.chain.getFinalizedHead()
    );

    const lastBlockNumber = signedBlock.block.header.number.toNumber();
    const lastBlockTime = getBlockTime(signedBlock);

    const firstBlockTime = lastBlockTime - 2 * 60 * 60 * 1000;
    debug(`Searching for the block at: ${new Date(firstBlockTime)}`);
    const firstBlockNumber = (await limiter.wrap(fetchHistoricBlockNum)(
      context.polkadotApi,
      lastBlockNumber,
      firstBlockTime
    )) as number;

    const length = lastBlockNumber - firstBlockNumber;
    const blockNumArray = Object.freeze(Array.from({ length }, (_, i) => firstBlockNumber + i));
    const limits = context.polkadotApi.consts.system.blockWeights;

    const getLimits = async (blockNum: number) => {
      const blockHash = await context.polkadotApi.rpc.chain.getBlockHash(blockNum);
      const apiAt = await context.polkadotApi.at(blockHash);
      const specVersion = apiAt.consts.system.version.specVersion.toNumber();

      if (specVersion >= 1700) {
        const { normal, operational } = await apiAt.query.system.blockWeight();
        return {
          hash: blockHash.toString(),
          weights: {
            normal,
            operational,
          },
        };
      }
    };

    const results = await Promise.all(
      blockNumArray.map((num) => limiter.schedule(() => getLimits(num)))
    );

    context.storeMemo("blockLimits", {
      normal: new BN(limits.perClass.normal.maxTotal.toJSON() as number),
      operational: new BN(limits.perClass.operational.maxTotal.toJSON() as number),
    });
    context.storeMemo("blockWeights", results);
    context.storeMemo("blockNumArray", blockNumArray);
  });

  // Normal class
  it("normal usage should be less than normal dispatch class limits", async function () {
    const blockLimits = context.getMemo("blockLimits");
    const blockWeights = context.getMemo("blockWeights");
    for (const block of blockWeights) {
      let used = block.weights.normal;
      let allowed = blockLimits.normal;
      expect(used.lte(allowed)).to.be.eq(
        true,
        `${block.hash} normal usage above allowed. Used ${used} and allowed ${allowed}.`
      );
    }
    debug(`Verified normal dispatch class`);
  });

  // Operational class
  it("operational usage should be less than operational dispatch class limits", async function () {
    const blockLimits = context.getMemo("blockLimits");
    const blockWeights = context.getMemo("blockWeights");
    for (const block of blockWeights) {
      let used = block.weights.operational;
      let allowed = blockLimits.operational;
      expect(used.lte(allowed)).to.be.eq(
        true,
        `${block.hash} operational usage above allowed. Used ${used} and allowed ${allowed}.`
      );
    }
    debug(`Verified operational dispatch class`);
  });

  // This will test that when Block is 20%+ full, its normal weight is mostly explained
  // by eth signed transactions.
  it("should roughly have a block weight mostly composed of transactions", async function () {
    this.timeout(120000);
    const blockNumArray = context.getMemo("blockNumArray");
    debug(
      `Checking #${blockNumArray[0]} - #${
        blockNumArray[blockNumArray.length - 1]
      } block weight proportions.`
    );

    const checkBlockWeight = async (blockNum: number) => {
      const apiAt = await context.polkadotApi.at(
        await context.polkadotApi.rpc.chain.getBlockHash(blockNum)
      );

      const normalWeight = (await apiAt.query.system.blockWeight()).normal.toNumber();
      const maxWeight = apiAt.consts.system.blockWeights.perClass.normal.maxTotal.toString();
      const ethBlock = (await apiAt.query.ethereum.currentBlock()).unwrap();

      const actualWeightUsed = normalWeight / Number(maxWeight);
      if (actualWeightUsed > 0.2) {
        const gasUsed = ethBlock.header.gasUsed.toBigInt();
        const weightCalc = gasUsed * WEIGHT_PER_GAS;
        const newRatio = (normalWeight - Number(weightCalc)) / Number(maxWeight);
        if (newRatio > 0.2) {
          debug(
            `Block #${blockNum} is ${(actualWeightUsed * 100).toFixed(2)}% full with ${
              ethBlock.transactions.length
            } transactions, non-transaction weight: ${(newRatio * 100).toFixed(2)}%`
          );
        }
        return { blockNum, nonTxn: newRatio };
      }
    };

    const results = await Promise.all(
      blockNumArray.map((num) => limiter.schedule(() => checkBlockWeight(num)))
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
    const blockNumArray = context.getMemo("blockNumArray");
    debug(
      `Checking if #${blockNumArray[0]} - #${
        blockNumArray[blockNumArray.length - 1]
      } extrinsic weights sum up.`
    );

    const checkWeights = async (blockNum: number) => {
      const hash = await context.polkadotApi.rpc.chain.getBlockHash(blockNum);
      const apiAt = await context.polkadotApi.at(hash);
      const events = await apiAt.query.system.events();

      const signedExtTotal = events
        .filter((a) => a.event.method == "ExtrinsicSuccess" || a.event.method == "ExtrinsicFailed")
        .filter((a) => (a.event.data as any).dispatchInfo.class.toString() != "Mandatory")
        .reduce((acc, curr) => acc + (curr.event.data as any).dispatchInfo.weight.toNumber(), 0);

      const normalWeights = (await apiAt.query.system.blockWeight()).normal.toNumber();
      const difference = (normalWeights - signedExtTotal) / signedExtTotal;
      if (difference > 0.2) {
        debug(
          `Block #${blockNum} signed extrinsic weight - reported: ${signedExtTotal},  accounted: ${normalWeights} (${
            difference > 0 ? "+" : "-"
          }${(difference * 100).toFixed(2)}%).`
        );
      }
      return { blockNum, signedExtTotal, normalWeights, difference };
    };

    const promises = blockNumArray.map((num) => limiter.schedule(() => checkWeights(num)));
    const results = await Promise.all(promises);
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
    const blockNumArray = context.getMemo("blockNumArray");
    debug(
      `Checking if #${blockNumArray[0]} - #${
        blockNumArray[blockNumArray.length - 1]
      } weights match gasUsed`
    );

    const compareGasToWeight = async (blockNum: number) => {
      const hash = await context.polkadotApi.rpc.chain.getBlockHash(blockNum);
      const apiAt = await context.polkadotApi.at(hash);
      const allEvents = await apiAt.query.system.events();
      const signedBlock = await context.polkadotApi.rpc.chain.getBlock(hash);
      const gasUsed = (await apiAt.query.ethereum.currentBlock())
        .unwrap()
        .header.gasUsed.toNumber();

      const gasWeight = gasUsed * Number(WEIGHT_PER_GAS);
      const ethTxnsWeight = signedBlock.block.extrinsics
        .map((item, index) => {
          if (item.method.method == "transact" && item.method.section == "ethereum") {
            return allEvents
              .filter(({ phase }) => phase.isApplyExtrinsic && phase.asApplyExtrinsic.eq(index))
              .filter(
                ({ event }) =>
                  (event.method == "ExtrinsicSuccess" && event.section == "system") ||
                  (event.method == "ExtrinsicFailed" && event.section == "system")
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
          `Block #${blockNum} has a ${((difference / ethTxnsWeight) * 100).toFixed(
            2
          )}% discrepancy between eth gas used and weight charged. `
        );
      }
      return { blockNum, gasWeight, ethTxnsWeight, difference };
    };

    const results = await Promise.all(
      blockNumArray.map((num) => limiter.schedule(() => compareGasToWeight(num)))
    );
    const discrepancies = results.filter((a) => a.difference > 0);
    expect(
      discrepancies,
      `These blocks have mismatching gas used vs charged weight, please investigate: ${discrepancies
        .map((a) => a.blockNum)
        .join(", ")}`
    ).to.be.empty;
  });

  // TODO: WRite a test to make sure there are no duplicate hashes in block weights map
  // TODO: Create a combined results array to further speed it up (do in is before all)
});

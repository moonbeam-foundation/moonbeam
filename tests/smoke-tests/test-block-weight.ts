import "@moonbeam-network/api-augment/moonbase";
import { BN } from "@polkadot/util";
import { expect } from "chai";
import { describeSmokeSuite } from "../util/setup-smoke-tests";
import Bottleneck from "bottleneck";
import { ethers } from "ethers";
import { EXTRINSIC_GAS_LIMIT, EXTRINSIC_BASE_WEIGHT, WEIGHT_PER_GAS } from "../util/constants";
import { fetchHistoricBlockNum, getBlockTime } from "../util/block";

const debug = require("debug")("smoke:block-weight");
const wssUrl = process.env.WSS_URL || null;
const relayWssUrl = process.env.RELAY_WSS_URL || null;
const ethRpcUrl = process.env.ETH_URL || null;

interface BlockWeights {
  hash: string;
  weights: BlockLimits;
}

interface BlockLimits {
  normal: BN;
  operational: BN;
}

describeSmokeSuite(
  `Verify weights of blocks being produced`,
  { wssUrl, relayWssUrl, ethRpcUrl },
  (context) => {

    const limiter = new Bottleneck({ maxConcurrent: 5 });
    let firstBlockNumber: number
    let lastBlockNumber: number

    before("Retrieve past hour's worth of blocks", async function () {
      const signedBlock = await context.polkadotApi.rpc.chain.getBlock(
        await context.polkadotApi.rpc.chain.getFinalizedHead()
      );

      lastBlockNumber = signedBlock.block.header.number.toNumber();
      const lastBlockTime = getBlockTime(signedBlock);

      // Target time here is set to be 1 hours, possible parameterize this in future
      const firstBlockTime = lastBlockTime - 5 * 60 * 60 * 1000;
      debug(`Searching for the block at: ${new Date(firstBlockTime)}`);
      firstBlockNumber = (await limiter.wrap(fetchHistoricBlockNum)(
        context.polkadotApi,
        lastBlockNumber,
        firstBlockTime
      )) as number;
    });


    // Despite being a naive test, this will flag up any egregiously heavy blocks in prod for 
    // further inspection
    it("should roughly have a block weight mostly composed of transactions", async function () {
      this.timeout(120000);
      debug(
        `Checking if #${firstBlockNumber} - #${lastBlockNumber} block weights.`
      );

      const checkBlockWeight = async (blockNum: number) => {
        /////// Check that if a Block is heavy, that it is because of extrinsics
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
          debug(
            `Block #${blockNum} is ${(actualWeightUsed * 100).toFixed(2)}% full with ${
              ethBlock.transactions.length
            } transactions, non-transaction weight: ${(newRatio * 100).toFixed(2)}%`
          );
          return { blockNum, nonTxn: newRatio };
        }
      };

      const promises = (() => {
        const length = lastBlockNumber - firstBlockNumber;
        return Array.from({ length }, (_, i) => firstBlockNumber + i);
      })().map((num) => limiter.schedule(() => checkBlockWeight(num)));

      const results = await Promise.all(promises);
      const nonTxnHeavyBlocks = results.filter((a) => a && a.nonTxn > 0.2);
      expect(
        nonTxnHeavyBlocks,
        `These blocks have non-txn weights >20%, please investigate: ${nonTxnHeavyBlocks
          .map((a) => a.blockNum)
          .join(", ")}`
      ).to.be.empty;
    });
  }

  /// TODO: Write new test for looking at sum of weights by looking at events extrinsic success
  /// and fails compared to normal block weight


);

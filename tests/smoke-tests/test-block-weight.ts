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



describeSmokeSuite(
  `Verify weights of blocks being produced`,
  { wssUrl, relayWssUrl, ethRpcUrl },
  (context) => {
    const limiter = new Bottleneck({ maxConcurrent: 5 });
    let blockArray: number[];

    before("Retrieve past hour's worth of blocks", async function () {
      const signedBlock = await context.polkadotApi.rpc.chain.getBlock(
        await context.polkadotApi.rpc.chain.getFinalizedHead()
      );

      const lastBlockNumber = signedBlock.block.header.number.toNumber();
      const lastBlockTime = getBlockTime(signedBlock);

      // Target time here is set to be 1 hours, possible parameterize this in future
      const firstBlockTime = lastBlockTime - 60 * 60 * 1000;
      debug(`Searching for the block at: ${new Date(firstBlockTime)}`);
      const firstBlockNumber = (await limiter.wrap(fetchHistoricBlockNum)(
        context.polkadotApi,
        lastBlockNumber,
        firstBlockTime
      )) as number;

      const length = lastBlockNumber - firstBlockNumber;
      blockArray = Array.from({ length }, (_, i) => firstBlockNumber + i);
    });

    // Despite being a naive test, this will flag up any egregiously heavy blocks in prod for
    // further inspection
    it("should roughly have a block weight mostly composed of transactions", async function () {
      this.timeout(120000);
      debug(
        `Checking #${blockArray[0]} - #${
          blockArray[blockArray.length - 1]
        } block weight proportions.`
      );

      const checkBlockWeight = async (blockNum: number) => {
        /////// Check that if a Block is heavy, that it is because of eth transactions
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

      const promises = blockArray.map((num) => limiter.schedule(() => checkBlockWeight(num)));

      const results = await Promise.all(promises);
      const nonTxnHeavyBlocks = results.filter((a) => a && a.nonTxn > 0.2);
      expect(
        nonTxnHeavyBlocks,
        `These blocks have non-txn weights >20%, please investigate: ${nonTxnHeavyBlocks
          .map((a) => a.blockNum)
          .join(", ")}`
      ).to.be.empty;
    });

    /// TODO: Write new test for looking at sum of weights by looking at events extrinsic success
    /// and fails compared to normal block weight

    it("should have a total weight charged matching sum of all extrinsics", async function () {
      this.timeout(120000);
      debug(
        `Checking if #${blockArray[0]} - #${
          blockArray[blockArray.length - 1]
        } extrinsic weights sum up.`
      );

      const BASE_BLOCK_WEIGHT = context.polkadotApi.consts.system.blockWeights.baseBlock.toNumber()

      const checkNonExtWeight = async (blockNum: number) => {
        const hash = await context.polkadotApi.rpc.chain.getBlockHash(blockNum);
        const apiAt = await context.polkadotApi.at(hash);
        const events = await apiAt.query.system.events();

        const extTotal = events
          .filter(
            (a) => a.event.method == "ExtrinsicSuccess" || a.event.method == "ExtrinsicFailed"
          )
          .reduce((acc, curr) => acc + (curr.event.data as any).dispatchInfo.weight.toNumber(), 0);

        const xcmpTotal = events
          .filter((a) => a.event.section == "xcmpQueue" && a.event.method == "Success")
          .reduce((acc, curr) => acc + (curr.event.data as any).weight.toNumber(), 0);

        const blockWeight = (await apiAt.query.system.blockWeight())
          .toArray()
          .reduce((total, current) => total + Number(current.toString()), 0);
        const ext = extTotal + xcmpTotal;
        const nonExt = blockWeight - extTotal - xcmpTotal - BASE_BLOCK_WEIGHT;
        debug(
          `Block #${blockNum} weights - non-ext: ${nonExt}, ext: ${ext}, total: ${blockWeight}.`
        );
        return { blockNum, ext, nonExt, blockWeight };
      };

      const promises = blockArray.map((num) => limiter.schedule(() => checkNonExtWeight(num)));

      const results = await Promise.all(promises);
      const heavyNonExts = results.filter(a=> a && (a.nonExt)> 0)
      console.log(heavyNonExts)
    });
  }
);

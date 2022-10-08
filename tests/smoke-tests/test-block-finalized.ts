import "@moonbeam-network/api-augment/moonbase";
import { expect } from "chai";
import { checkBlockFinalized, getBlockTime } from "../util/block";
import { describeSmokeSuite } from "../util/setup-smoke-tests";
import pLimit from "p-limit";
const debug = require("debug")("smoke:block-finalized");
const wssUrl = process.env.WSS_URL || null;
const relayWssUrl = process.env.RELAY_WSS_URL || null;
const ethRpcUrl = process.env.ETH_URL || null;

describeSmokeSuite(
  `Parachain blocks should be finalized..`,
  { wssUrl, relayWssUrl, ethRpcUrl },
  (context) => {
    it("should have a recently finalized block", async function () {
      const head = await context.polkadotApi.rpc.chain.getFinalizedHead();
      const block = (await context.polkadotApi.rpc.chain.getBlock(head)).toHuman() as any;
      const diff = Date.now() - getBlockTime(block);
      debug(`Last finalized block was ${diff / 1000} seconds ago`);
      expect(diff).to.be.lessThanOrEqual(10 * 60 * 1000); // 10 minutes in milliseconds
    });

    // When 0.9.29 is live we can enable this test
    // TODO: update the CI to provide ETH_URL env var
    it.skip("should have a recently finalized eth block", async function () {
      const timestamp = (await context.ethers.getBlock("finalized")).timestamp;
      const diff = Date.now() - timestamp * 1000;
      debug(`Last finalized block was ${diff / 1000} seconds ago`);
      expect(diff).to.be.lessThanOrEqual(10 * 60 * 1000);
    });

    it.only("should have only finalized blocks in the past two hours.", async function () {
      this.slow(10000);

      const finalHash = await context.polkadotApi.rpc.chain.getFinalizedHead();
      const signedBlock = await context.polkadotApi.rpc.chain.getBlock(finalHash);
      const lastBlockNumber = signedBlock.block.header.number.toNumber();
      const lastBlockTime = getBlockTime(signedBlock);
      const limit = pLimit(5);

      // Target time here is set to be 2 hours, possible parameterize this in future
      const firstBlockTime = lastBlockTime - 2 * 60 * 60 * 1000;

      const fetchBlockTime = async (blockNum) => {
        const hash = await context.polkadotApi.rpc.chain.getBlockHash(blockNum);
        const block = await context.polkadotApi.rpc.chain.getBlock(hash);
        return getBlockTime(block);
      };

      const fetchHistoricBlockNum = async (blockNumber, targetTime) => {
        return fetchBlockTime(blockNumber).then(async (time) => {
          if (time < targetTime) {
            return blockNumber;
          } else {
            return fetchHistoricBlockNum(
              (blockNumber -= Math.ceil((time - targetTime) / 30_000)),
              targetTime
            );
          }
        });
      };

      const firstBlockNumber = await fetchHistoricBlockNum(lastBlockNumber, firstBlockTime);

      // const firrrrstBlockNumber = (async function(blockNumber, targetTime){
      //   function fetchHistoricBlockNum(blockNumber, targetTime){
      //     fetchBlockTime(blockNumber).then(async (time) => {
      //       if (time < targetTime) {
      //         return blockNumber;
      //       } else {
      //         return fetchHistoricBlockNum((blockNumber -= Math.ceil((time - targetTime) / 30_000)),targetTime);
      //       }
      //     })
      //   }
      //   return fetchHistoricBlockNum(blockNumber, targetTime)
      // })()

      // console.log(await firrrrstBlockNumber)
      // console.log(JSON.stringify( await firrrrstBlockNumber))

      debug(`Checking if blocks #${firstBlockNumber} - #${lastBlockNumber} are finalized.`);

      const promises = range(firstBlockNumber, lastBlockNumber).map((num) =>
        limit(() => checkBlockFinalized(context.polkadotApi, num))
      );

      const results = await Promise.all(promises);
      results.forEach((item) => {
        if (!item.finalized) debug(`Historic block #${item.number} is unfinalized!`);
      });
      expect(results.every((item) => item.finalized)).to.be.true;
    });

    const range = (start, end) => {
      const length = end - start;
      return Array.from({ length }, (_, i) => start + i);
    };
  }
);

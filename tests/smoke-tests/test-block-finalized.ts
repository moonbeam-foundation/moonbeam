import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { describeSmokeSuite } from "../util/setup-smoke-tests";

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

    it("should have only finalized blocks in the past two hours.", async function () {
      this.timeout(120000);
      this.slow();

      const finalHash = await context.polkadotApi.rpc.chain.getFinalizedHead();
      const block = (await context.polkadotApi.rpc.chain.getBlock(finalHash)).toHuman() as any;
      const lastBlockNumber = Number(block.block.header.number.replace(/,/g, ""));
      const lastBlockTime = getBlockTime(block);

      // Target time here is set to be 2 hours, possible parameterize this in future
      const firstBlockTime = lastBlockTime - 2 * 60 * 60 * 1000;
      const firstBlockNumber = await fetchHistoricBlock(lastBlockNumber, firstBlockTime);
      debug(`Checking if blocks #${firstBlockNumber} - #${lastBlockNumber} are finalized.`);

      const promiseArray = range(firstBlockNumber, lastBlockNumber).map(async (num) => {
        //@ts-ignore typescript doesn't like the custom RPC methods
        const promise = await context.polkadotApi.rpc.moon.isBlockFinalized(
          await context.polkadotApi.rpc.chain.getBlockHash(num)
        );
        return { number: num, finalized: promise };
      });

      await Promise.all(promiseArray).then((results) => {
        results.forEach((item) => {
          if (item.finalized.isFalse) debug(`Historic block #${item.number} is unfinalized!`);
        });
        expect(results.every((item) => item.finalized.isTrue)).to.be.true;
      });
    });

    const fetchBlockTime = async (blockNum) => {
      const hash = await context.polkadotApi.rpc.chain.getBlockHash(blockNum);
      const block = (await context.polkadotApi.rpc.chain.getBlock(hash)).toHuman() as any;
      return getBlockTime(block);
    };

    const getBlockTime = (block: any): number => {
      const item = block.block.extrinsics.find((item) => item.method.section == "timestamp");
      return Number(item.method.args.now.replace(/,/g, ""));
    };

    const range = (start, end) => {
      const length = end - start;
      return Array.from({ length }, (_, i) => start + i);
    };

    const fetchHistoricBlock = async (blockNumber, targetTime) => {
      return await fetchBlockTime(blockNumber).then(async (time) => {
        if (time < targetTime) {
          return blockNumber;
        } else {
          return await fetchHistoricBlock(
            (blockNumber -= Math.ceil((time - targetTime) / 40_000)),
            targetTime
          );
        }
      });
    };
  }
);

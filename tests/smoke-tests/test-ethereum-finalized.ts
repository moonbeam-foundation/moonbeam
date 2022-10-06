import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { describeSmokeSuite } from "../util/setup-smoke-tests";

const debug = require("debug")("smoke:ethereum-finalized");

const wssUrl = process.env.WSS_URL || null;
const relayWssUrl = process.env.RELAY_WSS_URL || null;

describeSmokeSuite(`Parachain blocks should be finalized..`, { wssUrl, relayWssUrl }, (context) => {
  it("should have a recent finalized block", async function () {
    const head = await context.polkadotApi.rpc.chain.getFinalizedHead();
    const block = (await context.polkadotApi.rpc.chain.getBlock(head)).toHuman() as any;
    const item = block.block.extrinsics.find((item) => item.method.section == "timestamp");
    const value = item.method.args.now.replace(/,/g, "");
    const diff = Date.now() - value;
    debug(`Last finalized block was ${diff / 1000} seconds ago`);
    // Check last block is within 10 minutes of current time
    expect(diff).to.be.lessThanOrEqual(10 * 60 * 1000);
  });

  it("should have only finalized blocks in the past two hours ", async function () {
    this.timeout(120000);
    // At time of writing the average blocktime is 12.955s
    const blockTime = 12.95;
    const finalHash = await context.polkadotApi.rpc.chain.getFinalizedHead();
    const lastBlockNumber = (
      await context.polkadotApi.rpc.chain.getHeader(finalHash)
    ).number.toNumber();

    // 2 hours worth of blocks assuming average blocktime
    const firstBlockNumber = Math.floor(lastBlockNumber - (2 * 60 * 60) / blockTime);
    debug(`Checking blocks #${firstBlockNumber} - #${lastBlockNumber} are finalized.`);

    const array = [];
    for (let i = firstBlockNumber; i <= lastBlockNumber; i++) {
      //@ts-ignore typescript doesn't like these custom RPC methods
      const promise = await context.polkadotApi.rpc.moon.isBlockFinalized(
        await context.polkadotApi.rpc.chain.getBlockHash(i)
      );
      array.push({ number: i, finalized: promise });
    }

    await Promise.all(array).then((results) => {
      results.forEach((item) => {
        if (item.finalized == false) debug(`Historic block #${item.number} is unfinalized!`);
      });
      expect(results.every((item) => item.finalized == true)).to.be.true;
    });
  });
});

import "@moonbeam-network/api-augment/moonbase";
import { expect } from "chai";
import { checkBlockFinalized, getBlockTime, fetchHistoricBlockNum } from "../util/block";
import { describeSmokeSuite } from "../util/setup-smoke-tests";
import Bottleneck from "bottleneck";
const debug = require("debug")("smoke:block-finalized");
const timePeriod = process.env.TIME_PERIOD ? Number(process.env.TIME_PERIOD) : 2 * 60 * 60 * 1000;

describeSmokeSuite(`Parachain blocks should be finalized..`, (context) => {
  it("should have a recently finalized block", async function () {
    const head = await context.polkadotApi.rpc.chain.getFinalizedHead();
    const block = await context.polkadotApi.rpc.chain.getBlock(head);
    const diff = Date.now() - getBlockTime(block);
    debug(`Last finalized block was ${diff / 1000} seconds ago`);
    expect(diff).to.be.lessThanOrEqual(10 * 60 * 1000); // 10 minutes in milliseconds
  });

  it("should have a recently finalized eth block", async function () {
    const specVersion = context.polkadotApi.consts.system.version.specVersion.toNumber();
    if (specVersion < 1900) {
      debug(`ChainSpec ${specVersion} does not support Finalized BlockTag, skipping test`);
      this.skip();
    }

    const timestamp = (await context.ethers.getBlock("finalized")).timestamp;
    const diff = Date.now() - timestamp * 1000;
    debug(`Last finalized eth block was ${diff / 1000} seconds ago`);
    expect(diff).to.be.lessThanOrEqual(10 * 60 * 1000);
  });

  it(`should have only finalized blocks in the past ${(timePeriod / (1000 * 60 * 60)).toFixed(
    2
  )} hours`, async function () {
    this.timeout(120000);
    const signedBlock = await context.polkadotApi.rpc.chain.getBlock(
      await context.polkadotApi.rpc.chain.getFinalizedHead()
    );

    const lastBlockNumber = signedBlock.block.header.number.toNumber();
    const lastBlockTime = getBlockTime(signedBlock);
    const limiter = new Bottleneck({ maxConcurrent: 5 });

    const firstBlockTime = lastBlockTime - timePeriod;
    debug(`Searching for the block at: ${new Date(firstBlockTime)}`);

    const firstBlockNumber = (await limiter.wrap(fetchHistoricBlockNum)(
      context.polkadotApi,
      lastBlockNumber,
      firstBlockTime
    )) as number;

    debug(`Checking if blocks #${firstBlockNumber} - #${lastBlockNumber} are finalized.`);

    const promises = (() => {
      const length = lastBlockNumber - firstBlockNumber;
      return Array.from({ length }, (_, i) => firstBlockNumber + i);
    })().map((num) => limiter.schedule(() => checkBlockFinalized(context.polkadotApi, num)));

    const results = await Promise.all(promises);

    const unfinalizedBlocks = results.filter((item) => !item.finalized);
    expect(
      unfinalizedBlocks,
      `The following blocks were not finalized ${unfinalizedBlocks.map((a) => a.number).join(", ")}`
    ).to.be.empty;
  });
});

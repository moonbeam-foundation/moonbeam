import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { describeDevMoonbeam } from "../../util/setup-dev-tests";

describeDevMoonbeam("Block 2", (context) => {
  before("Setup: Create 2 empty blocks", async () => {
    await context.createBlock();
    await context.createBlock();
  });

  it("should be at block 2", async function () {
    expect(await context.web3.eth.getBlockNumber()).to.equal(2);
  });

  it("should include previous block hash as parent", async function () {
    const block = await context.web3.eth.getBlock("latest");
    let previousBlock = await context.web3.eth.getBlock(1);
    expect(block.hash).to.not.equal(previousBlock.hash);
    expect(block.parentHash).to.equal(previousBlock.hash);
  });
});

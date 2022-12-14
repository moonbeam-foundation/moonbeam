import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { alith } from "../../util/accounts";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";

describeDevMoonbeam("Block 1", (context) => {
  before("Setup: Create empty block", async () => {
    await context.createBlock();
  });

  it("should be at block 1", async function () {
    expect(await context.web3.eth.getBlockNumber()).to.equal(1);
  });

  it("should have valid timestamp after block production", async function () {
    // Originally ,this test required the timestamp be in the last finve minutes.
    // This requirement doesn't make sense when we forge timestamps in manual seal.
    const block = await context.web3.eth.getBlock("latest");
    const next5Minutes = Date.now() / 1000 + 300;
    expect(block.timestamp).to.be.least(0);
    expect(block.timestamp).to.be.below(next5Minutes);
  });

  it("should contain block information", async function () {
    const block = await context.web3.eth.getBlock("latest");
    expect(block).to.include({
      author: alith.address.toLocaleLowerCase(), // web3 doesn't checksum
      difficulty: "0",
      extraData: "0x",
      gasLimit: 15000000,
      gasUsed: 0,
      logsBloom: `0x${"0".repeat(512)}`,
      miner: "0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac",
      number: 1,
      receiptsRoot: "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
      sha3Uncles: "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
      totalDifficulty: "0",
      transactionsRoot: "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
    });

    expect(block.transactions).to.be.a("array").empty;
    expect(block.uncles).to.be.a("array").empty;
    expect(block.nonce).to.be.eq("0x0000000000000000");
    expect(block.hash).to.be.a("string").lengthOf(66);
    expect(block.parentHash).to.be.a("string").lengthOf(66);
    expect(block.timestamp).to.be.a("number");
  });

  it("should be accessible by hash", async function () {
    const latestBlock = await context.web3.eth.getBlock("latest");
    const block = await context.web3.eth.getBlock(latestBlock.hash);
    expect(block.hash).to.be.eq(latestBlock.hash);
  });

  it("should be accessible by number", async function () {
    const latestBlock = await context.web3.eth.getBlock("latest");
    const block = await context.web3.eth.getBlock(1);
    expect(block.hash).to.be.eq(latestBlock.hash);
  });
});

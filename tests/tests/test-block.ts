import { expect } from "chai";
import { step } from "mocha-steps";
import { contractCreation, GENESIS_ACCOUNT } from "./constants";

import { createAndFinalizeBlock, describeWithMoonbeam, fillBlockWithTx } from "./util";

describeWithMoonbeam("Moonbeam RPC (Block)", `simple-specs.json`, (context) => {
  let previousBlock;
  // Those tests are dependant of each other in the given order.
  // The reason is to avoid having to restart the node each time
  // Running them individually will result in failure

  step("should be at block 0 at genesis", async function () {
    expect(await context.web3.eth.getBlockNumber()).to.equal(0);
  });

  step("should return genesis block", async function () {
    expect(await context.web3.eth.getBlockNumber()).to.equal(0);
    const block = await context.web3.eth.getBlock(0);
    expect(block).to.include({
      author: "0x0000000000000000000000000000000000000000",
      difficulty: "0",
      extraData: "0x",
      gasLimit: 15000000,
      gasUsed: 0,
      logsBloom: `0x${"0".repeat(512)}`,
      number: 0,
      receiptsRoot: "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
      sha3Uncles: "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
      totalDifficulty: "0",
      transactionsRoot: "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
    });

    expect(block.transactions).to.be.a("array").empty;
    expect(block.uncles).to.be.a("array").empty;
    expect((block as any).sealFields).to.eql([
      "0x0000000000000000000000000000000000000000000000000000000000000000",
      "0x0000000000000000",
    ]);
    expect(block.hash).to.be.a("string").lengthOf(66);
    expect(block.parentHash).to.be.a("string").lengthOf(66);
    expect(block.timestamp).to.be.a("number");
  });

  // TODO: unskip this when https://github.com/paritytech/frontier/pull/279 is merged
  it.skip("fetch genesis block by hash", async function () {
    //fetch block again using hash
    const block = await context.web3.eth.getBlock(0);
    const blockByHash = await context.web3.eth.getBlock(block.hash);
    console.log("blockbyhash", blockByHash);
    expect(blockByHash).to.include({
      author: "0x0000000000000000000000000000000000000000",
      difficulty: "0",
      extraData: "0x",
      gasLimit: 4294967295,
      gasUsed: 0,
      logsBloom: `0x${"0".repeat(512)}`,
      number: 0,
      receiptsRoot: "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
      sha3Uncles: "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
      totalDifficulty: "0",
      transactionsRoot: "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
    });
  });

  let firstBlockCreated = false;
  step("should be at block 1 after block production", async function () {
    this.timeout(15000);
    await createAndFinalizeBlock(context.polkadotApi);
    expect(await context.web3.eth.getBlockNumber()).to.equal(1);
    firstBlockCreated = true;
  });

  step("should have valid timestamp after block production", async function () {
    // Originally ,this test required the timestamp be in the last finve minutes.
    // This requirement doesn't make sense when we forge timestamps in manual seal.
    const block = await context.web3.eth.getBlock("latest");
    const next5Minutes = Date.now() / 1000 + 300;
    expect(block.timestamp).to.be.least(0);
    expect(block.timestamp).to.be.below(next5Minutes);
  });

  step("retrieve block information", async function () {
    expect(firstBlockCreated).to.be.true;

    const block = await context.web3.eth.getBlock("latest");
    expect(block).to.include({
      author: "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b",
      difficulty: "0",
      extraData: "0x",
      gasLimit: 15000000,
      gasUsed: 0,
      //hash: "0x14fe6f7c93597f79b901f8b5d7a84277a90915b8d355959b587e18de34f1dc17",
      logsBloom: `0x${"0".repeat(512)}`,
      miner: "0x6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b",
      number: 1,
      //parentHash: "0x04540257811b46d103d9896e7807040e7de5080e285841c5430d1a81588a0ce4",
      receiptsRoot: "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
      sha3Uncles: "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
      totalDifficulty: "0",
      //transactions: [],
      transactionsRoot: "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
      //uncles: []
    });
    previousBlock = block;

    expect(block.transactions).to.be.a("array").empty;
    expect(block.uncles).to.be.a("array").empty;
    expect((block as any).sealFields).to.eql([
      "0x0000000000000000000000000000000000000000000000000000000000000000",
      "0x0000000000000000",
    ]);
    expect(block.hash).to.be.a("string").lengthOf(66);
    expect(block.parentHash).to.be.a("string").lengthOf(66);
    expect(block.timestamp).to.be.a("number");
  });

  step("get block by hash", async function () {
    const latest_block = await context.web3.eth.getBlock("latest");
    const block = await context.web3.eth.getBlock(latest_block.hash);
    expect(block.hash).to.be.eq(latest_block.hash);
  });

  step("get block by number", async function () {
    const block = await context.web3.eth.getBlock(1);
    expect(block).not.null;
  });

  step("should include previous block hash as parent (block 2)", async function () {
    this.timeout(15000);
    await createAndFinalizeBlock(context.polkadotApi);
    const block = await context.web3.eth.getBlock("latest");
    expect(block.hash).to.not.equal(previousBlock.hash);
    expect(block.parentHash).to.equal(previousBlock.hash);
  });

  // tx/block tests

  step("genesis balance enough to make all the transfers", async function () {
    expect(Number(await context.web3.eth.getBalance(GENESIS_ACCOUNT))).to.gte(512 * 100000);
  });

  // the maximum number of tx/ blocks is not constant but is always around 1500

  it("should be able to fill a block with a 1 tx", async function () {
    this.timeout(15000);
    let { txPassedFirstBlock } = await fillBlockWithTx(context, 1);
    expect(txPassedFirstBlock).to.eq(1);
  });

  it.skip("should be able to fill a block with 260 tx", async function () {
    this.timeout(15000);
    // We have 6_000_000 Gas available for transactions per block.
    // Each transaction needs 2_000 (extrinsic cost) + 21_000 (eth cost)
    // 6_000_000 / 23_000 = ~260.86
    // The test will send 261 tx and verify the first block contains only 260.
    let { txPassed, txPassedFirstBlock } = await fillBlockWithTx(context, 261);
    expect(txPassedFirstBlock).to.eq(260);
    expect(txPassed).to.eq(261); // including all blocks
  });

  it.skip("should be able to fill a block with 64 contract creations tx", async function () {
    this.timeout(15000);
    // We have 6_000_000 Gas available for transactions per block.
    // Each transaction needs 2_000 (extrinsic cost) + 91019 (contract cost)
    // 6_000_000 / 92_019 = ~64.50

    // The test will send 65 contract tx and verify the first block contains only 64.
    let { txPassedFirstBlock } = await fillBlockWithTx(context, 65, contractCreation);
    expect(txPassedFirstBlock).to.eq(64);
  });

  // 8192 is the number of tx that can be sent to the Pool
  // before it throws an error and drops all tx

  it.skip("should be able to send 8192 tx to the pool and have them all published\
  within the following blocks", async function () {
    this.timeout(120000);
    let { txPassed } = await fillBlockWithTx(context, 8192);
    expect(txPassed).to.eq(8192);
  });

  it.skip("but shouldn't work for 8193", async function () {
    this.timeout(120000);
    let { txPassed } = await fillBlockWithTx(context, 8193);
    expect(txPassed).to.eq(0);
  });

  it.skip("should be able to send 8192 tx to the pool and have them all published\
  within the following blocks - bigger tx", async function () {
    this.timeout(120000);
    let { txPassed } = await fillBlockWithTx(context, 8192, contractCreation);
    expect(txPassed).to.eq(8192);
  });

  it.skip("but shouldn't work for 8193 - bigger tx", async function () {
    this.timeout(120000);
    let { txPassed } = await fillBlockWithTx(context, 8193, contractCreation);
    expect(txPassed).to.eq(0);
  });
});

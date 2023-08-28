import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { describeDevMoonbeam } from "../../util/setup-dev-tests";

describeDevMoonbeam("Block genesis", (context) => {
  it("should be at block 0", async function () {
    expect(await context.web3.eth.getBlockNumber()).to.equal(0);
  });

  it("should contain block details", async function () {
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
    //fetch block again using hash
    const block = await context.web3.eth.getBlock(0);
    const blockByHash = await context.web3.eth.getBlock(block.hash);
    expect(blockByHash).to.include({
      author: "0x0000000000000000000000000000000000000000",
      difficulty: "0",
      extraData: "0x",
      gasLimit: 15000000,
      gasUsed: 0,
      logsBloom: `0x${"0".repeat(512)}`,
      number: 0,
      receiptsRoot: "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
      sha3Uncles: "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
      totalDifficulty: "0",
      transactionsRoot: "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
    });
  });
});

import { expect } from "chai";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createTransfer } from "../util/transactions";

import { TEST_ACCOUNT } from "../util/constants";

describeDevMoonbeam("Fork", (context) => {
  it("should change best chain to the longest chain", async function () {
    // Creation of the best chain so far, with blocks 0-1-2
    await context.createBlock({ finalize: false });
    await context.createBlock({ finalize: false });

    // Lets grab the ethereum block hashes so far
    let ethHash1 = (await context.web3.eth.getBlock(1)).hash;
    let ethHash2 = (await context.web3.eth.getBlock(2)).hash;

    // Now lets fork the chain
    let currentHeight = await context.web3.eth.getBlockNumber();
    // We start parenting to the genesis
    let parentHash = await context.polkadotApi.rpc.chain.getBlockHash(0);
    for (let i = 0; i <= currentHeight; i++) {
      parentHash = (await context.createBlock({ parentHash, finalize: false })).block.hash;
    }

    // We created at 1 block more than the previous best chain. We should be in the best chain now
    // Ethereum blocks should have changed
    expect(await context.web3.eth.getBlockNumber()).to.equal(currentHeight + 1);
    expect((await context.web3.eth.getBlock(1)).hash).to.not.equal(ethHash1);
    expect((await context.web3.eth.getBlock(2)).hash).to.not.equal(ethHash2);
  });
});

describeDevMoonbeam("Fork", (context) => {
  it("should re-insert Tx from retracted fork on new canonical chain", async function () {
    // Creation of the best chain so far, with blocks 0-1-2 and a transfer in block 2
    await context.createBlock({ finalize: false });
    const { txResults } = await context.createBlock({
      finalize: false,
      transactions: [await createTransfer(context.web3, TEST_ACCOUNT, 512)],
    });
    const insertedTx = txResults[0].result;
    let retractedTx = await context.web3.eth.getTransaction(insertedTx)
    expect(retractedTx).to.not.be.null;

    // Fork 4 blocks 0-1-2-3-4
    let parentHash = await context.polkadotApi.rpc.chain.getBlockHash(0);
    parentHash = (await context.createBlock({ parentHash, finalize: false })).block.hash;
    parentHash = (await context.createBlock({ parentHash, finalize: false })).block.hash;
    // This next block defines a new best chain. Substrate will insert the retracted block's txs in the tx pool
    parentHash = (await context.createBlock({ parentHash, finalize: false })).block.hash;
    // All the transactions that were in the pool are now on-chain
    parentHash = (await context.createBlock({ parentHash, finalize: true })).block.hash;
    let finalTx = await context.web3.eth.getTransaction(insertedTx);
    let currentHeight = await context.web3.eth.getBlockNumber();
    // The Tx should have been inserted in the last block
    expect(finalTx.blockNumber).to.equal(currentHeight);
    // The Tx should have the hash of the latest canonical chain
    expect(finalTx.blockHash).to.not.equal(retractedTx.blockHash);
    expect(finalTx.blockHash).to.equal((await context.web3.eth.getBlock(currentHeight)).hash);
  });
});

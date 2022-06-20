import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { baltathar } from "../../util/accounts";
import { describeDevMoonbeam, describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import { createTransfer } from "../../util/transactions";

describeDevMoonbeam("Chain - Fork", (context) => {
  it("should change best chain to the longest chain", async function () {
    // Creation of the best chain so far, with blocks 0-1-2
    await context.createBlock([], { finalize: false });
    await context.createBlock([], { finalize: false });

    // Lets grab the ethereum block hashes so far
    let ethHash1 = (await context.web3.eth.getBlock(1)).hash;
    let ethHash2 = (await context.web3.eth.getBlock(2)).hash;

    // Now lets fork the chain
    let currentHeight = await context.web3.eth.getBlockNumber();
    // We start parenting to the genesis
    let parentHash = (await context.polkadotApi.rpc.chain.getBlockHash(0)) as any;
    for (let i = 0; i <= currentHeight; i++) {
      parentHash = (await context.createBlock([], { parentHash, finalize: false })).block.hash;
    }

    // We created at 1 block more than the previous best chain. We should be in the best chain now
    // Ethereum blocks should have changed
    expect(await context.web3.eth.getBlockNumber()).to.equal(currentHeight + 1);
    expect((await context.web3.eth.getBlock(1)).hash).to.not.equal(ethHash1);
    expect((await context.web3.eth.getBlock(2)).hash).to.not.equal(ethHash2);
  });
});

describeDevMoonbeamAllEthTxTypes("Chain - Fork", (context) => {
  it("should re-insert Tx from retracted fork on new canonical chain", async function () {
    // Creation of the best chain so far, with blocks 0-1-2 and a transfer in block 2
    await context.createBlock([], { finalize: false });
    const { result } = await context.createBlock(createTransfer(context, baltathar.address, 512), {
      finalize: false,
    });
    const insertedTx = result.hash;
    const retractedTx = await context.web3.eth.getTransaction(insertedTx);
    expect(retractedTx).to.not.be.null;

    // Fork from 0-1-2
    //      to   0-1b-2b-3b-4b-5b-6b-7b-8b-9b-10b
    let parentHash = (await context.polkadotApi.rpc.chain.getBlockHash(0)) as any;
    // Create enough blocks to ensure the TX is re-scheduled and that chain is new best
    for (let i = 0; i < 10; i++) {
      parentHash = (await context.createBlock([], { parentHash, finalize: false })).block.hash;
      // TODO: investigate why ! Gives extra time (trouble with ci)
      await new Promise((resolve) => setTimeout(resolve, 100));
    }
    const finalTx = await context.web3.eth.getTransaction(insertedTx);
    // The Tx should have been inserted in the new best chain
    expect(finalTx.blockHash).to.not.be.eq(retractedTx.blockHash);
  });
});

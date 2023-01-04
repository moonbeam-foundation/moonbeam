import "@moonbeam-network/api-augment";

import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { alith, generateKeyringPair } from "../../util/accounts";
import { BLOCK_TX_GAS_LIMIT, GLMR, MICROGLMR } from "../../util/constants";
import { expectOk } from "../../util/expect";
import { expect } from "chai";
import { BlockForkEvent } from "@ethersproject/abstract-provider";

describeDevMoonbeam("TxPool - Limits", (context) => {
  it.skip("should be able to fill a block with 260 tx", async function () {});

  it.skip("should be able to fill a block with 64 contract creations tx", async function () {});

  // txpool size is 8192
  it("should be able drain 8192 txns", async function () {
    this.timeout(1000000);

    // constants used below, we will do two rounds of NUM_TXNS txns
    const NUM_TXNS = 8192;
    const BATCH_SIZE = 512;
    const NUM_BATCHES = NUM_TXNS / BATCH_SIZE;
    expect(NUM_TXNS % BATCH_SIZE).to.eq(0);

    // step 1: create 8192 accounts with some balances. this is so that each txn we later send to
    // the pool can come from a different account and not have any inter-txn dependencies (which
    // could impact performance)
    console.log(`generating and funding ${NUM_TXNS} accounts...`);
    let accounts = [];
    for (let i=0; i<NUM_TXNS; i++) {
      const randomAccount = generateKeyringPair();
      accounts[i] = randomAccount;
    }

    for (let i=0; i<NUM_BATCHES; i++) {
      for (let a=0; a<BATCH_SIZE; a++) {
        const index = i * BATCH_SIZE + a;
        await context.polkadotApi.tx.balances
          .transfer(accounts[index].address, 10n * GLMR)
          .signAndSend(alith, { nonce: index });
      }
    }

    console.log(`txns sent, waiting for blocks...`);

    let numTxnsFound = 0;
    let numBlocks = 0;
    let blockWeights = [];
    while (numTxnsFound < NUM_TXNS) {
      const result = await context.createBlock();
      numBlocks += 1;
      const hash = result.block.hash;

      const apiAt = await context.polkadotApi.at(hash);
      const [{ block }, events] = await Promise.all([
        context.polkadotApi.rpc.chain.getBlock(hash),
        apiAt.query.system.events(),
      ]);
      block.extrinsics.forEach((ext, index) => {
        if (ext.signer.toHex() === alith.address.toLowerCase()) {
          // console.log(`Found alith txn (${numTxnsFound+1})`);
          numTxnsFound += 1;
        }
      });

      blockWeights.push((await (await apiAt.query.system.blockWeight()).normal.refTime));
    }

    console.log(`Found all txns in ${numBlocks} blocks`);
    console.log(`Block weights: ${blockWeights.reduce((prev, curr) => prev+"\n"+curr, "\n")}`);

    /*
    // ensure all accounts have tokens (TODO: remove, this is expensive)
    console.log(`verifying account balances...`);
    for (let i=0; i<NUM_TXNS; i++) {
      // console.log(`checking balance for ${i} ${accounts[i].address}`)
      const account = await context.polkadotApi.query.system.account(accounts[i].address);
      expect((account as any).data.free.toBigInt()).to.equal(10n * GLMR);
    }
    */

    // now send a transfer from each account back to alith
    // using a unique tip each time
    // ---------------------------------------------------
    console.log(`sending transfers from each account (generating txns)...`);

    for (let i=0; i<NUM_BATCHES; i++) {
      for (let a=0; a<BATCH_SIZE; a++) {
        const index = i * BATCH_SIZE + a;
        const tip = BigInt(index) * MICROGLMR;
        await context.polkadotApi.tx.balances
          .transfer(alith.address, 1n * MICROGLMR)
          .signAndSend(accounts[index], { nonce: 0, tip });
      }
    }

    console.log(`sending transfers from each account (creating blocks)...`);

    numTxnsFound = 0;
    numBlocks = 0;
    blockWeights = [];
    while (numTxnsFound < NUM_TXNS) {
      // console.log(`creating block (so far: ${numTxnsFound})...`);
      const result = await context.createBlock();
      numBlocks += 1;
      const hash = result.block.hash;

      const apiAt = await context.polkadotApi.at(hash);
      const [{ block }, events] = await Promise.all([
        context.polkadotApi.rpc.chain.getBlock(hash),
        apiAt.query.system.events(),
      ]);
      block.extrinsics.forEach((ext, index) => {
        // console.log(`ext method: ${ext.method}`)
        if (ext.signer.toHex() !== "0x0000000000000000000000000000000000000000") {
          // console.log(`ext method: ${ext}`);
          numTxnsFound += 1;
        }
      });
      // numTxnsFound = NUM_TXNS; // XXX: remove

      blockWeights.push((await (await apiAt.query.system.blockWeight()).normal.refTime));
    }

    console.log(`Found all txns in ${numBlocks} blocks`);
    console.log(`Block weights: ${blockWeights.reduce((prev, curr) => prev+"\n"+curr, "\n")}`);

  });

  it.skip("shouldn't work for 8193", async function () {});

  it.skip("should be able to send 8192 tx to the pool and have them all published\
    within the following blocks - bigger tx", async function () {});

  it.skip("shouldn't work for 8193 - bigger tx", async function () {});
});

import { expect } from "chai";

import { createAndFinalizeBlock, describeWithMoonbeam, customRequest } from "./util";

import { GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY, TEST_ACCOUNT } from "./constants";

describeWithMoonbeam("Frontier RPC (fork)", `simple-specs.json`, (context) => {
  let inserted_tx;
  before(async function () {
    this.timeout(15000);
    // Creation of the best chain so far, with blocks 0-1-2 and a transfer in block 2
    await createAndFinalizeBlock(
      context.polkadotApi,
      await context.polkadotApi.rpc.chain.getBlockHash(0),
      false
    );
    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        to: TEST_ACCOUNT,
        value: "0x200", // Must be higher than ExistentialDeposit (currently 0)
        gasPrice: "0x01",
        gas: "0x100000",
      },
      GENESIS_ACCOUNT_PRIVATE_KEY
    );

    let send = await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);
    inserted_tx = send.result;
    await createAndFinalizeBlock(
      context.polkadotApi,
      await context.polkadotApi.rpc.chain.getBlockHash(1),
      false
    );
  });
  it("Should create another best chain, finalize it and do sanity checks", async function () {
    // Lets grab the ethereum block hashes so far
    let eth_hash_1 = (await context.web3.eth.getBlock(1)).hash;
    let eth_hash_2 = (await context.web3.eth.getBlock(2)).hash;

    // Now lets fork the chain
    let current_height = await context.web3.eth.getBlockNumber();
    // We start parenting to the genesis
    let parent_hash = await context.polkadotApi.rpc.chain.getBlockHash(0);

    for (var i = 0; i <= current_height; i++) {
      parent_hash = (await createAndFinalizeBlock(context.polkadotApi, parent_hash, false))[1];
    }

    // We created at 1 block more than the previous best chain. We should be in the best chain now
    // Ethereum blocks should have changed
    // The previous inserted transaction should dissapear
    expect(await context.web3.eth.getBlockNumber()).to.equal(current_height + 1);
    expect((await context.web3.eth.getBlock(1)).hash).to.not.equal(eth_hash_1);
    expect((await context.web3.eth.getBlock(2)).hash).to.not.equal(eth_hash_2);
    expect(await context.web3.eth.getTransaction(inserted_tx)).to.be.null;
  });
});

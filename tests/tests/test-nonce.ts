import { expect } from "chai";
import { step } from "mocha-steps";

import { createAndFinalizeBlock, describeWithMoonbeam, customRequest } from "./util";
import { Keyring } from "@polkadot/keyring";
import { GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY, TEST_ACCOUNT } from "./constants";

describeWithMoonbeam("Moonbeam RPC (Nonce)", `simple-specs.json`, (context) => {
  step("get nonce", async function () {
    this.timeout(10_000);
    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        to: TEST_ACCOUNT,
        value: "0x200",
        gasPrice: "0x01",
        gas: "0x100000",
      },
      GENESIS_ACCOUNT_PRIVATE_KEY
    );

    expect(await context.web3.eth.getTransactionCount(GENESIS_ACCOUNT, "earliest")).to.eq(0);

    await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);

    expect(await context.web3.eth.getTransactionCount(GENESIS_ACCOUNT, "latest")).to.eq(0);
    expect(await context.web3.eth.getTransactionCount(GENESIS_ACCOUNT, "pending")).to.eq(1);

    await createAndFinalizeBlock(context.polkadotApi);

    expect(await context.web3.eth.getTransactionCount(GENESIS_ACCOUNT, "latest")).to.eq(1);
    expect(await context.web3.eth.getTransactionCount(GENESIS_ACCOUNT, "pending")).to.eq(1);
    expect(await context.web3.eth.getTransactionCount(GENESIS_ACCOUNT, "earliest")).to.eq(0);
  });

  it("nonce should not be reset to 0 when emptying dust accounts", async function () {
    this.timeout(15000);

    const testAccountPrivateKey1 = context.web3.utils.randomHex(32);
    const testAccountPrivateKey2 = context.web3.utils.randomHex(32);
    const keyring = new Keyring({ type: "ethereum" });
    const testAccount1 = await keyring.addFromUri(testAccountPrivateKey1, null, "ethereum");
    const testAccount2 = await keyring.addFromUri(testAccountPrivateKey2, null, "ethereum");
    const genesisAccount = await keyring.addFromUri(GENESIS_ACCOUNT_PRIVATE_KEY, null, "ethereum");

    const info = await context.polkadotApi.tx.balances
      .transfer(testAccount1.address, 1)
      .paymentInfo(genesisAccount);

    // We should estimate the fee to ensure we are transferring enough funds
    const fee = info.partialFee.toNumber();

    await context.polkadotApi.tx.balances
      .transfer(testAccount1.address, fee + 1)
      .signAndSend(genesisAccount);
    await createAndFinalizeBlock(context.polkadotApi);

    await context.polkadotApi.tx.balances
      .transfer(testAccount2.address, 1)
      .signAndSend(testAccount1);

    await createAndFinalizeBlock(context.polkadotApi);

    const { nonce, data: balance } = await context.polkadotApi.query.system.account(
      testAccount1.address
    );

    expect(nonce.toNumber()).to.equal(1);
    expect(balance.free.toNumber()).to.equal(0);
  });
});

import { expect } from "chai";
import { step } from "mocha-steps";

import { createAndFinalizeBlock, describeWithMoonbeam, customRequest } from "./util";

describeWithMoonbeam("Frontier RPC (Balance)", `simple-specs.json`, (context) => {
  const GENESIS_ACCOUNT = "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b";
  const GENESIS_ACCOUNT_BALANCE = "340282366920938463463374607431768211455";
  const GENESIS_ACCOUNT_PRIVATE_KEY =
    "0x99B3C12287537E38C90A9219D4CB074A89A16E9CDB20BF85728EBD97C343E342";
  const TEST_ACCOUNT = "0x1111111111111111111111111111111111111111";

  step("genesis balance is setup correctly (web3)", async function () {
    expect(await context.web3.eth.getBalance(GENESIS_ACCOUNT)).to.equal(GENESIS_ACCOUNT_BALANCE);
  });
  step("genesis balance is setup correctly (polkadotJs)", async function () {
    const account = await context.polkadotApi.query.system.account(GENESIS_ACCOUNT);
    console.log(account.data.free.toHex());
    console.log(account.data.free.toHuman());
    expect(account.data.free.toString()).to.equal(GENESIS_ACCOUNT_BALANCE);
  });

  step("balance to be updated after transfer", async function () {
    this.timeout(15000);

    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        to: TEST_ACCOUNT,
        value: "0x200", // Must me higher than ExistentialDeposit (500)
        gasPrice: "0x01",
        gas: "0x100000",
      },
      GENESIS_ACCOUNT_PRIVATE_KEY
    );
    await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);
    await createAndFinalizeBlock(context.web3);
    expect(await context.web3.eth.getBalance(GENESIS_ACCOUNT)).to.equal(
      "340282366920938463463374607431768189943"
    );
    expect(await context.web3.eth.getBalance(TEST_ACCOUNT)).to.equal("512");
  });

  step("balance should be the same on polkadot/web3", async function () {
    this.timeout(15000);

    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        to: TEST_ACCOUNT,
        value: "0x200", // Must me higher than ExistentialDeposit (500)
        gasPrice: "0x01",
        gas: "0x100000",
      },
      GENESIS_ACCOUNT_PRIVATE_KEY
    );
    await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);
    await createAndFinalizeBlock(context.web3);
    expect(await context.web3.eth.getBalance(GENESIS_ACCOUNT)).to.equal(
      "340282366920938463463374607431768189943"
    );
    expect(await context.web3.eth.getBalance(TEST_ACCOUNT)).to.equal("512");
  });
});

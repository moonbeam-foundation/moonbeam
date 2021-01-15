import { expect } from "chai";
import { step } from "mocha-steps";

import { describeWithMoonbeam, createAndFinalizeBlock } from "./util";

describeWithMoonbeam("Moonbeam RPC (Stake)", `simple-specs.json`, (context) => {
  const GENESIS_ACCOUNT = "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b";
  const GENESIS_STAKED = "100000";
  const ACCOUNT_BALANCE = "340282366920938463463374607431768111455";
  step("validator bond reserved in genesis", async function () {
    const account = await context.polkadotApi.query.system.account(GENESIS_ACCOUNT);
    expect(account.data.reserved.toString()).to.equal(GENESIS_STAKED);
  });

  step("validator set in genesis", async function () {
    const validators = await context.polkadotApi.query.stake.validators();
    expect((validators[0] as Buffer).toString("hex").toLowerCase()).equal(GENESIS_ACCOUNT);
  });

  step("issuance minted to the sole validator for authoring blocks", async function () {
    const expectedBalance = BigInt(ACCOUNT_BALANCE) + BigInt(49);
    const expectedBalance2 = expectedBalance + BigInt(49);

    var block = await context.web3.eth.getBlockNumber();
    while (block < 40) {
      await createAndFinalizeBlock(context.polkadotApi);
      block = await context.web3.eth.getBlockNumber();
    }
    expect(await context.web3.eth.getBalance(GENESIS_ACCOUNT)).to.equal(expectedBalance.toString());
    while (block < 60) {
      await createAndFinalizeBlock(context.polkadotApi);
      block = await context.web3.eth.getBlockNumber();
    }
    expect(await context.web3.eth.getBalance(GENESIS_ACCOUNT)).to.equal(
      expectedBalance2.toString()
    );
  });
});

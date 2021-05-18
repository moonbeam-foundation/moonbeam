import { Keyring } from "@polkadot/api";
import { KeyringPair } from "@polkadot/keyring/types";
import { expect } from "chai";

import {
  DEFAULT_GENESIS_STAKING,
  GENESIS_ACCOUNT,
  GENESIS_ACCOUNT_PRIVATE_KEY,
  TEST_ACCOUNT,
} from "../util/constants";
import { describeDevMoonbeam } from "../util/setup-dev-tests";

describeDevMoonbeam("Crowdloan", (context) => {
  let genesisAccount: KeyringPair;
  const relayChainAddress: string = "couldBeAnyString";
  before("Setup genesis account for substrate", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    genesisAccount = await keyring.addFromUri(GENESIS_ACCOUNT_PRIVATE_KEY, null, "ethereum");
    // await context.polkadotApi.tx.balances.transfer(testAccount, 123).signAndSend(genesisAccount);
    // await context.createBlock();
  });
  it("should be able to register the genesis account for reward", async function () {
    const isPayable = await context.polkadotApi.query.crowdloanRewards.accountsPayable(
      GENESIS_ACCOUNT
    );
    // console.log("ACCOUNT",isPayable.toHuman(), isPayable.createdAtHash)
    expect(isPayable.toHuman()).to.equal(null);
    // const isPayable2 = await context.polkadotApi.query.crowdloanRewards.accountsPayable(TEST_ACCOUNT);
    // console.log("ACCOUNT2",isPayable2.toHuman())
    //expect(account.data.reserved.toString()).to.equal(DEFAULT_GENESIS_STAKING.toString());
    await context.polkadotApi.tx.sudo
      .sudo(
        context.polkadotApi.tx.crowdloanRewards.initializeRewardVec(
          [[relayChainAddress, GENESIS_ACCOUNT, 500000]], // use 1000 and should work too
          1,
          0,
          1
        )
      )
      .signAndSend(genesisAccount);
    await context.createBlock();
    const isPayable3 = await context.polkadotApi.query.crowdloanRewards.accountsPayable(
      GENESIS_ACCOUNT
    );
    // console.log("ACCOUNT",isPayable3.toHuman(), isPayable3.createdAtHash)
    expect((isPayable3.toHuman() as any).total_reward).to.equal("500.0000 fUnit");
    // TODO: add claimed_reward test
  });
  it("should show me the money", async function () {
    await context.polkadotApi.tx.sudo
      .sudo(context.polkadotApi.tx.crowdloanRewards.showMeTheMoney())
      .signAndSend(genesisAccount);
    await context.createBlock();
    const isPayable4 = await context.polkadotApi.query.crowdloanRewards.accountsPayable(
      GENESIS_ACCOUNT
    );
    console.log("ACCOUNT", isPayable4.toHuman());
    expect((isPayable4.toHuman() as any).claimed_reward).to.equal("2.0000 aUnit");
    // const isPayable5 = await context.polkadotApi.query.crowdloanRewards.VestingPeriod(
    //   GENESIS_ACCOUNT
    // );
    // console.log("ACCOUNT", isPayable5.toHuman());
  });
  it("should show me the money after 5 blocks", async function () {
    await context.createBlock();
    await context.createBlock();
    await context.createBlock();
    await context.polkadotApi.tx.sudo
      .sudo(context.polkadotApi.tx.crowdloanRewards.showMeTheMoney())
      .signAndSend(genesisAccount);
    await context.createBlock();
    const isPayable4 = await context.polkadotApi.query.crowdloanRewards.accountsPayable(
      GENESIS_ACCOUNT
    );
    console.log("ACCOUNT", isPayable4.toHuman());
    expect((isPayable4.toHuman() as any).claimed_reward).to.equal("6.0000 aUnit");
    // const isPayable5 = await context.polkadotApi.query.crowdloanRewards.VestingPeriod(
    //   GENESIS_ACCOUNT
    // );
    // console.log("ACCOUNT", isPayable5.toHuman());
  });
});

describeDevMoonbeam(
  "should be able to register the genesis account for reward - with multiplier",
  (context) => {
    let genesisAccount: KeyringPair;
    const relayChainAddress: string = "couldBeAnyString";
    before("Setup genesis account for substrate", async () => {
      const keyring = new Keyring({ type: "ethereum" });
      genesisAccount = await keyring.addFromUri(GENESIS_ACCOUNT_PRIVATE_KEY, null, "ethereum");
      // await context.polkadotApi.tx.balances.transfer(testAccount, 123).signAndSend(genesisAccount);
      // await context.createBlock();
    });
    it("should be able to register the genesis account for reward - with multiplier", async function () {
      console.log(
        "ha",
        (
          await context.polkadotApi.query.crowdloanRewards.accountsPayable(GENESIS_ACCOUNT)
        ).toHuman()
      );
      expect(
        (
          await context.polkadotApi.query.crowdloanRewards.accountsPayable(GENESIS_ACCOUNT)
        ).toHuman()
      ).to.equal(null);
      await context.polkadotApi.tx.sudo
        .sudo(
          context.polkadotApi.tx.crowdloanRewards.initializeRewardVec(
            [[relayChainAddress, GENESIS_ACCOUNT, 500]],
            1000,
            0,
            1
          )
        )
        .signAndSend(genesisAccount);
      await context.createBlock();
      expect(
        (
          (
            await context.polkadotApi.query.crowdloanRewards.accountsPayable(GENESIS_ACCOUNT)
          ).toHuman() as any
        ).total_reward
      ).to.equal("500.0000 fUnit");
    });
  }
);

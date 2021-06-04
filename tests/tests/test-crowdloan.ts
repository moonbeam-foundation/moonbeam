import { Keyring } from "@polkadot/api";
import { KeyringPair } from "@polkadot/keyring/types";
import { expect } from "chai";
import Web3 from "web3";
import { Account } from "web3-core";
import { formatBalance } from "@polkadot/util";

import {
  GENESIS_ACCOUNT,
  GENESIS_ACCOUNT_BALANCE,
  GENESIS_ACCOUNT_PRIVATE_KEY,
  GLMR,
  ALITH_PRIV_KEY,
  ALITH,
} from "../util/constants";
import { describeDevMoonbeam } from "../util/setup-dev-tests";

async function calculate_vested_amount(context, totalReward, initialPayment, numberOfBlocks) {
  let vesting = (await context.polkadotApi.consts.crowdloanRewards.vestingPeriod).toString() as any;
  let amountToVest = BigInt(totalReward) - BigInt(initialPayment);
  let vestedPerBlock = amountToVest / BigInt(vesting);

  // On average a parachain only gets a candidate into every other relay chain block.
  // In the dev service, where the relay block number is mocked, we get exactly two relay blocks.
  let elapsedRelayBlocks = numberOfBlocks * 2;
  let shouldHaveVested = BigInt(initialPayment) + vestedPerBlock * BigInt(elapsedRelayBlocks);
  let claimedAsBalance = formatBalance(shouldHaveVested, { withSi: true, withUnit: "Unit" }, 18);
  return claimedAsBalance;
}

describeDevMoonbeam("Crowdloan", (context) => {
  let genesisAccount: KeyringPair, sudoAccount: KeyringPair;
  const relayChainAddress: string = "couldBeAnyString";
  before("Setup genesis account for substrate", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    genesisAccount = await keyring.addFromUri(GENESIS_ACCOUNT_PRIVATE_KEY, null, "ethereum");
    sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
  });
  it("should check initial state", async function () {
    // check that genesis has genesis balance
    expect(Number(await context.web3.eth.getBalance(GENESIS_ACCOUNT))).to.eq(
      Number(GENESIS_ACCOUNT_BALANCE)
    );
    // check that genesis is not registered
    const isPayable = await context.polkadotApi.query.crowdloanRewards.accountsPayable(
      GENESIS_ACCOUNT
    );
    expect(isPayable.toHuman()).to.equal(null);
    // check vesting period
    expect((await context.polkadotApi.consts.crowdloanRewards.vestingPeriod).toHuman()).to.eq(
      "201,600"
    );
  });
  it("should be able to register the genesis account for reward", async function () {
    // should be able to register the genesis account for reward
    await context.polkadotApi.tx.sudo
      .sudo(
        context.polkadotApi.tx.crowdloanRewards.initializeRewardVec(
          [[relayChainAddress, GENESIS_ACCOUNT, 3_000_000n * GLMR]],
          0,
          1
        )
      )
      .signAndSend(sudoAccount);
    await context.createBlock();
    expect(
      (
        (
          await context.polkadotApi.query.crowdloanRewards.accountsPayable(GENESIS_ACCOUNT)
        ).toHuman() as any
      ).total_reward
    ).to.equal("3.0000 MUnit");
    let isInitialized = await context.polkadotApi.query.crowdloanRewards.initialized();
    expect(isInitialized.toHuman()).to.be.true;
  });
});

describeDevMoonbeam("Crowdloan", (context) => {
  let genesisAccount: KeyringPair, sudoAccount: KeyringPair;
  const relayChainAddress: string = "couldBeAnyString";
  before("Setup genesis account for substrate", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    genesisAccount = await keyring.addFromUri(GENESIS_ACCOUNT_PRIVATE_KEY, null, "ethereum");
    sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
  });
  it("should be able to make a first claim", async function () {
    await context.polkadotApi.tx.sudo
      .sudo(
        context.polkadotApi.tx.crowdloanRewards.initializeRewardVec(
          [[relayChainAddress, GENESIS_ACCOUNT, 3_000_000n * GLMR]],
          0,
          1
        )
      )
      .signAndSend(sudoAccount);
    await context.createBlock();

    let rewardInfo = (
      await context.polkadotApi.query.crowdloanRewards.accountsPayable(GENESIS_ACCOUNT)
    ).toJSON() as any;
    let claimed = await calculate_vested_amount(
      context,
      rewardInfo.total_reward,
      rewardInfo.claimed_reward,
      1
    );
    // construct a transaction
    const transfer = context.polkadotApi.tx.crowdloanRewards.claim();
    await transfer.signAndSend(genesisAccount);

    await context.createBlock();

    expect(
      (
        (
          await context.polkadotApi.query.crowdloanRewards.accountsPayable(GENESIS_ACCOUNT)
        ).toHuman() as any
      ).claimed_reward
    ).to.equal(claimed);

    // check balances
    expect(
      (
        Number(await context.web3.eth.getBalance(GENESIS_ACCOUNT)) - Number(GENESIS_ACCOUNT_BALANCE)
      ).toString()
    ).to.equal("6.0002380012380946e+23");
    const account = await context.polkadotApi.query.system.account(GENESIS_ACCOUNT);
    expect(
      (Number(account.data.free.toString()) - Number(GENESIS_ACCOUNT_BALANCE)).toString()
    ).to.equal("6.0002380012380946e+23");
  });
});

describeDevMoonbeam("Crowdloan", (context) => {
  let genesisAccount: KeyringPair, sudoAccount: KeyringPair;
  const relayChainAddress: string = "couldBeAnyString";
  before("Setup genesis account for substrate", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    genesisAccount = await keyring.addFromUri(GENESIS_ACCOUNT_PRIVATE_KEY, null, "ethereum");
    sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
  });
  it("should show me the money after 5 blocks, after first claim was called", async function () {
    await context.polkadotApi.tx.sudo
      .sudo(
        context.polkadotApi.tx.crowdloanRewards.initializeRewardVec(
          [[relayChainAddress, GENESIS_ACCOUNT, 3_000_000n * GLMR]],
          0,
          1
        )
      )
      .signAndSend(sudoAccount);
    await context.createBlock();

    let rewardInfo = (
      await context.polkadotApi.query.crowdloanRewards.accountsPayable(GENESIS_ACCOUNT)
    ).toJSON() as any;

    // construct a transaction
    const transfer = context.polkadotApi.tx.crowdloanRewards.claim();
    await transfer.signAndSend(genesisAccount);

    await context.createBlock();

    // should show me the money after 5 blocks
    await context.createBlock();
    await context.createBlock();
    await context.createBlock();

    let claimed = await calculate_vested_amount(
      context,
      rewardInfo.total_reward,
      rewardInfo.claimed_reward,
      5
    );

    await context.polkadotApi.tx.crowdloanRewards.claim().signAndSend(genesisAccount);
    await context.createBlock();
    const isPayable4 = await context.polkadotApi.query.crowdloanRewards.accountsPayable(
      GENESIS_ACCOUNT
    );
    expect((isPayable4.toHuman() as any).claimed_reward).to.equal(claimed);
  });
});

describeDevMoonbeam("Crowdloan", (context) => {
  let genesisAccount: KeyringPair, sudoAccount: KeyringPair;
  const relayChainAddress: string = "couldBeAnyString";
  before("Setup genesis account for substrate", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    genesisAccount = await keyring.addFromUri(GENESIS_ACCOUNT_PRIVATE_KEY, null, "ethereum");
    sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
  });
  it("should make first claim 5 blocks after initialization called", async function () {
    await context.polkadotApi.tx.sudo
      .sudo(
        context.polkadotApi.tx.crowdloanRewards.initializeRewardVec(
          [[relayChainAddress, GENESIS_ACCOUNT, 3_000_000n * GLMR]],
          0,
          1
        )
      )
      .signAndSend(sudoAccount);
    await context.createBlock();
    let rewardInfo = (
      await context.polkadotApi.query.crowdloanRewards.accountsPayable(GENESIS_ACCOUNT)
    ).toJSON() as any;
    await context.createBlock();

    // should show me the money after 5 blocks
    await context.createBlock();
    await context.createBlock();
    await context.createBlock();
    let claimed = await calculate_vested_amount(
      context,
      rewardInfo.total_reward,
      rewardInfo.claimed_reward,
      5
    );

    await context.polkadotApi.tx.crowdloanRewards.claim().signAndSend(genesisAccount);
    await context.createBlock();

    const isPayable4 = await context.polkadotApi.query.crowdloanRewards.accountsPayable(
      GENESIS_ACCOUNT
    );
    expect((isPayable4.toHuman() as any).claimed_reward).to.equal(claimed);
  });
});

describeDevMoonbeam("Crowdloan", (context) => {
  let genesisAccount: KeyringPair, sudoAccount: KeyringPair;
  const relayChainAddress: string = "couldBeAnyString";
  before("Setup genesis account for substrate", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    genesisAccount = await keyring.addFromUri(GENESIS_ACCOUNT_PRIVATE_KEY, null, "ethereum");
    sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
  });
  it("should not be able to call initializeRewardVec another time", async function () {
    await context.polkadotApi.tx.sudo
      .sudo(
        context.polkadotApi.tx.crowdloanRewards.initializeRewardVec(
          [[relayChainAddress, GENESIS_ACCOUNT, 3_000_000n * GLMR]],
          0,
          1
        )
      )
      .signAndSend(sudoAccount);
    await context.createBlock();

    // should not be able to call initializeRewardVec another time
    await context.polkadotApi.tx.sudo
      .sudo(
        context.polkadotApi.tx.crowdloanRewards.initializeRewardVec(
          [[relayChainAddress, ALITH, 1000n * GLMR]],
          0,
          1
        )
      )
      .signAndSend(genesisAccount);
    await context.createBlock();
    expect(
      (await context.polkadotApi.query.crowdloanRewards.accountsPayable(ALITH)).toHuman() as any
    ).to.equal(null);
  });
});

describeDevMoonbeam("Crowdloan", (context) => {
  let genesisAccount: KeyringPair;
  let alithAccount: KeyringPair;

  const relayChainAddress: string = "couldBeAnyString";
  before("Setup genesis account for substrate", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    genesisAccount = await keyring.addFromUri(GENESIS_ACCOUNT_PRIVATE_KEY, null, "ethereum");
    alithAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
  });

  it("should be able to register the genesis account - with small amount", async function () {
    // initializeRewardVec
    await context.polkadotApi.tx.sudo
      .sudo(
        context.polkadotApi.tx.crowdloanRewards.initializeRewardVec(
          [[relayChainAddress, ALITH, 3_000_000n * GLMR]],
          0,
          1
        )
      )
      .signAndSend(alithAccount);
    await context.createBlock();
    expect(
      ((await context.polkadotApi.query.crowdloanRewards.accountsPayable(ALITH)).toHuman() as any)
        .total_reward
    ).to.equal("3.0000 MUnit");

    let rewardInfo = (
      await context.polkadotApi.query.crowdloanRewards.accountsPayable(ALITH)
    ).toJSON() as any;
    let claimed = await calculate_vested_amount(
      context,
      rewardInfo.total_reward,
      rewardInfo.claimed_reward,
      1
    );
    // claim
    await context.polkadotApi.tx.crowdloanRewards.claim().signAndSend(alithAccount);
    await context.createBlock();

    const isPayable4 = await context.polkadotApi.query.crowdloanRewards.accountsPayable(ALITH);
    expect((isPayable4.toHuman() as any).claimed_reward).to.equal(claimed);
  });
});

describeDevMoonbeam("Crowdloan", (context) => {
  let genesisAccount: KeyringPair, sudoAccount: KeyringPair;
  const relayChainAddress: string = "couldBeAnyString";
  const numberOfAccounts: number = 5000; // min 2
  let largInput: [string, string, bigint][];

  before("Setup genesis account for substrate", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    genesisAccount = await keyring.addFromUri(GENESIS_ACCOUNT_PRIVATE_KEY, null, "ethereum");
    sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
  });

  it("should be able to register many accounts : " + numberOfAccounts, async function () {
    // should create a bunch of test eth accounts
    this.timeout(30000);
    let web3 = new Web3();
    // We need to make sure the rewards match the account funds. 3M GLMR/ number of accounts
    let accounts = new Array(numberOfAccounts).fill(0).map((_, i) => web3.eth.accounts.create());
    largInput = accounts.map((acc: Account, i: number) => {
      return [
        relayChainAddress + i.toString(),
        acc.address,
        (3_000_000n * GLMR) / BigInt(numberOfAccounts),
      ];
    });
    expect(largInput.length).to.eq(numberOfAccounts);
    expect(largInput[0][1] !== largInput[numberOfAccounts - 1][1]).to.eq(true);

    // should be able to register many accounts
    await context.polkadotApi.tx.sudo
      .sudo(context.polkadotApi.tx.crowdloanRewards.initializeRewardVec(largInput, 0, largInput))
      .signAndSend(sudoAccount);
    await context.createBlock();
    await Promise.all(
      largInput.map(async (input) => {
        expect(
          (
            (
              await context.polkadotApi.query.crowdloanRewards.accountsPayable(input[1])
            ).toHuman() as any
          ).total_reward
        ).to.equal("600.0000 Unit");
      })
    );
  });
});

describeDevMoonbeam("Crowdloan", (context) => {
  let genesisAccount: KeyringPair, sudoAccount: KeyringPair;
  const relayChainAddress: string = "couldBeAnyString";
  const numberOfAccounts: number = 5000; // min 2
  let largInput: [string, string, bigint][];

  before("Setup genesis account for substrate", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    genesisAccount = await keyring.addFromUri(GENESIS_ACCOUNT_PRIVATE_KEY, null, "ethereum");
    sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
  });

  it("should be able to register many accounts - batch : " + numberOfAccounts, async function () {
    // should create a bunch of test eth accounts
    this.timeout(20000);
    let web3 = new Web3();
    let accounts = new Array(numberOfAccounts).fill(0).map((_, i) => web3.eth.accounts.create());
    largInput = accounts.map((acc: Account, i: number) => {
      return [
        relayChainAddress + i.toString(),
        acc.address,
        (3_000_000n * GLMR) / BigInt(numberOfAccounts),
      ];
    });
    expect(largInput.length).to.eq(numberOfAccounts);
    expect(largInput[0][1] !== largInput[numberOfAccounts - 1][1]).to.eq(true);

    // should be able to register many accounts
    await context.polkadotApi.tx.utility
      .batch([
        await context.polkadotApi.tx.sudo.sudo(
          context.polkadotApi.tx.crowdloanRewards.initializeRewardVec(
            largInput.slice(0, Math.floor(numberOfAccounts / 3)),
            0,
            3
          )
        ),
        await context.polkadotApi.tx.sudo.sudo(
          context.polkadotApi.tx.crowdloanRewards.initializeRewardVec(
            largInput.slice(
              Math.floor(numberOfAccounts / 3),
              Math.floor((numberOfAccounts * 2) / 3)
            ),
            1,
            3
          )
        ),
        await context.polkadotApi.tx.sudo.sudo(
          context.polkadotApi.tx.crowdloanRewards.initializeRewardVec(
            largInput.slice(Math.floor((numberOfAccounts * 2) / 3), numberOfAccounts),
            2,
            3
          )
        ),
      ])
      .signAndSend(sudoAccount);
    await context.createBlock();
    await Promise.all(
      largInput.map(async (input) => {
        expect(
          (
            (
              await context.polkadotApi.query.crowdloanRewards.accountsPayable(input[1])
            ).toHuman() as any
          ).total_reward
        ).to.equal("600.0000 Unit");
      })
    );
  });
});

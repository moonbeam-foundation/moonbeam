import { expect } from "chai";
import { Keyring } from "@polkadot/api";
import { KeyringPair } from "@polkadot/keyring/types";
import {
  GENESIS_ACCOUNT,
  ALITH_PRIV_KEY,
  GENESIS_ACCOUNT_PRIVATE_KEY,
  ZERO_ADDRESS,
  GENESIS_ACCOUNT_BALANCE,
  TEST_ACCOUNT,
  ALITH,
} from "../util/constants";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createBlockWithExtrinsic } from "../util/substrate-rpc";
import { verifyLatestBlockFees } from "../util/block";
import { createTransfer } from "../util/transactions";

describeDevMoonbeam("Sudo - successful setParachainBondAccount", (context) => {
  let alith: KeyringPair;
  before("Setup genesis account for substrate", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    alith = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
  });
  it("should be able to call sudo with the right account", async function () {
    const { events } = await createBlockWithExtrinsic(
      context,
      alith,
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.parachainStaking.setParachainBondAccount(GENESIS_ACCOUNT)
      )
    );
    //check parachainBondInfo
    const parachainBondInfo = await context.polkadotApi.query.parachainStaking.parachainBondInfo();
    expect(parachainBondInfo.toHuman()["account"]).to.equal(GENESIS_ACCOUNT);
    expect(parachainBondInfo.toHuman()["percent"]).to.equal("30.00%");
    //check events
    expect(events.length).to.eq(5);
    expect(context.polkadotApi.events.parachainStaking.ParachainBondAccountSet.is(events[1])).to.be
      .true;
    expect(context.polkadotApi.events.balances.Deposit.is(events[3])).to.be.true;
    expect(context.polkadotApi.events.system.ExtrinsicSuccess.is(events[4])).to.be.true;
    // check balance diff (diff should be null for sudo - funds are sent back)
    expect(await context.web3.eth.getBalance(GENESIS_ACCOUNT, 1)).to.equal(
      GENESIS_ACCOUNT_BALANCE.toString()
    );
  });
});
describeDevMoonbeam("Sudo - fail if no funds in sudo", (context) => {
  let alith: KeyringPair;
  before("Setup genesis account for substrate", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    alith = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
    const initBalance = await context.web3.eth.getBalance(ALITH);
    await context.createBlock({
      transactions: [
        await createTransfer(
          context,
          TEST_ACCOUNT,
          BigInt(initBalance) - 1n - 21000n * 1_000_000_000n,
          {
            from: ALITH,
            privateKey: ALITH_PRIV_KEY,
            gas: 21000,
            gasPrice: 1_000_000_000,
          }
        ),
      ],
    });
    console.log("aft", await context.web3.eth.getBalance(ALITH));
    expect(await context.web3.eth.getBalance(ALITH)).to.eq("1");
  });
  it("should not be able to call sudo with no funds", async function () {
    try {
      await createBlockWithExtrinsic(
        context,
        alith,
        context.polkadotApi.tx.sudo.sudo(
          context.polkadotApi.tx.parachainStaking.setParachainBondAccount(GENESIS_ACCOUNT)
        )
      );
    } catch (e) {
      expect(e.toString()).to.eq(
        "Error: -32000: Invalid Transaction: Inability " +
          "to pay some fees , e.g. account balance too low"
      );
    }
    //check parachainBondInfo
    const parachainBondInfo = await context.polkadotApi.query.parachainStaking.parachainBondInfo();
    expect(parachainBondInfo.toHuman()["account"]).to.equal(ZERO_ADDRESS);
  });
});
describeDevMoonbeam("Sudo - Only sudo account", (context) => {
  let genesisAccount: KeyringPair;
  before("Setup genesis account for substrate", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    genesisAccount = await keyring.addFromUri(GENESIS_ACCOUNT_PRIVATE_KEY, null, "ethereum");
  });
  it("should NOT be able to call sudo with another account than sudo account", async function () {
    const { events } = await createBlockWithExtrinsic(
      context,
      genesisAccount,
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.parachainStaking.setParachainBondAccount(GENESIS_ACCOUNT)
      )
    );
    //check parachainBondInfo
    const parachainBondInfo = await context.polkadotApi.query.parachainStaking.parachainBondInfo();
    expect(parachainBondInfo.toHuman()["account"]).to.equal(ZERO_ADDRESS);
    expect(parachainBondInfo.toHuman()["percent"]).to.equal("30.00%");
    //check events
    expect(events.length === 6).to.be.true;
    expect(context.polkadotApi.events.system.NewAccount.is(events[2])).to.be.true;
    expect(context.polkadotApi.events.balances.Endowed.is(events[3])).to.be.true;
    expect(context.polkadotApi.events.treasury.Deposit.is(events[4])).to.be.true;
    expect(context.polkadotApi.events.system.ExtrinsicFailed.is(events[5])).to.be.true;
    // check balance diff (should not be null for a failed extrinsic)
    expect(
      BigInt(await context.web3.eth.getBalance(GENESIS_ACCOUNT, 1)) - GENESIS_ACCOUNT_BALANCE !== 0n
    ).to.equal(true);
  });
});

describeDevMoonbeam("Sudo - Only sudo account - test gas", (context) => {
  let alith: KeyringPair;
  before("Setup genesis account for substrate", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    alith = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
  });
  it("should NOT be able to call sudo with another account than sudo account", async function () {
    await createBlockWithExtrinsic(
      context,
      alith,
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.parachainStaking.setParachainBondAccount(GENESIS_ACCOUNT)
      )
    );

    await verifyLatestBlockFees(context, expect);
  });
});

import { expect } from "chai";
import { Keyring } from "@polkadot/api";
import { KeyringPair } from "@polkadot/keyring/types";
import { Event } from "@polkadot/types/interfaces";
import {
  GENESIS_ACCOUNT,
  ALITH_PRIV_KEY,
  GENESIS_ACCOUNT_PRIVATE_KEY,
  ZERO_ADDRESS,
  GENESIS_ACCOUNT_BALANCE,
} from "../util/constants";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createBlockWithExtrinsic } from "../util/substrate-rpc";
import { verifyLatestBlockFees } from "../util/block";

describeDevMoonbeam("Sudo - Only sudo account", (context) => {
  let genesisAccount: KeyringPair;
  before("Setup genesis account for substrate", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    genesisAccount = await keyring.addFromUri(GENESIS_ACCOUNT_PRIVATE_KEY, null, "ethereum");
  });
  it.only("should NOT be able to call sudo with another account than sudo account", async function () {
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
    // check balance diff (shouold be null for sudo)
    console.log(
      "sgoodbal",
      GENESIS_ACCOUNT_BALANCE - BigInt(await context.web3.eth.getBalance(GENESIS_ACCOUNT, 0))
    );
    console.log(
      "diff",
      GENESIS_ACCOUNT_BALANCE - BigInt(await context.web3.eth.getBalance(GENESIS_ACCOUNT, 1))
    );
    expect(await context.web3.eth.getBalance(GENESIS_ACCOUNT, 1)).to.equal(
      GENESIS_ACCOUNT_BALANCE.toString()
    );
  });
});

describeDevMoonbeam("Sudo - Only sudo account - test gas", (context) => {
  let genesisAccount: KeyringPair;
  before("Setup genesis account for substrate", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    genesisAccount = await keyring.addFromUri(GENESIS_ACCOUNT_PRIVATE_KEY, null, "ethereum");
  });
  it.only("should NOT be able to call sudo with another account than sudo account", async function () {
    await createBlockWithExtrinsic(
      context,
      genesisAccount,
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.parachainStaking.setParachainBondAccount(GENESIS_ACCOUNT)
      )
    );

    await verifyLatestBlockFees(context.polkadotApi, expect);
  });
});

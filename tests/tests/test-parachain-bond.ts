import { expect } from "chai";
import { Keyring } from "@polkadot/api";
import { KeyringPair } from "@polkadot/keyring/types";
import {
  DEFAULT_GENESIS_MAPPING,
  DEFAULT_GENESIS_STAKING,
  GENESIS_ACCOUNT,
  COLLATOR_ACCOUNT,
  ALITH_PRIV_KEY,
  GENESIS_ACCOUNT_PRIVATE_KEY,
} from "../util/constants";
import { describeDevMoonbeam } from "../util/setup-dev-tests";

describeDevMoonbeam("Staking - Parachain Bond - genesis and setParachainBondAccount", (context) => {
  let sudoAccount: KeyringPair;
  before("Setup genesis account for substrate", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    // genesisAccount = await keyring.addFromUri(GENESIS_ACCOUNT_PRIVATE_KEY, null, "ethereum");
    sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
  });
  it("should have right parachain bond in genesis", async function () {
    const parachainBondInfo = await context.polkadotApi.query.parachainStaking.parachainBondInfo();
    console.log("parachainBondInfo", parachainBondInfo.toHuman());
    // const expectedReserved = DEFAULT_GENESIS_STAKING + DEFAULT_GENESIS_MAPPING;
    // expect(account.data.reserved.toString()).to.equal(expectedReserved.toString());
  });
  it("should be able set the parachain bond with sudo", async function () {
    // should be able to register the genesis account for reward
    await context.polkadotApi.tx.sudo
      .sudo(context.polkadotApi.tx.parachainStaking.setParachainBondAccount(GENESIS_ACCOUNT))
      .signAndSend(sudoAccount);
    await context.createBlock();
    const parachainBondInfo = await context.polkadotApi.query.parachainStaking.parachainBondInfo();
    console.log("parachainBondInfo", parachainBondInfo.toHuman());
  });
});

describeDevMoonbeam("Staking - Parachain Bond - no sudo", (context) => {
  let genesisAccount: KeyringPair, sudoAccount: KeyringPair;
  before("Setup genesis account for substrate", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    genesisAccount = await keyring.addFromUri(GENESIS_ACCOUNT_PRIVATE_KEY, null, "ethereum");
    sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
  });
  it("should NOT be able set the parachain bond with sudo", async function () {
    // should be able to register the genesis account for reward
    await context.polkadotApi.tx.parachainStaking
      .setParachainBondAccount(GENESIS_ACCOUNT)
      .signAndSend(genesisAccount);
    await context.createBlock();
    const parachainBondInfo = await context.polkadotApi.query.parachainStaking.parachainBondInfo();
    console.log("parachainBondInfo", parachainBondInfo.toHuman());
  });
});

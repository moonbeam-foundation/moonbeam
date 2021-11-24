import { expect } from "chai";
import { Keyring } from "@polkadot/api";
import { KeyringPair } from "@polkadot/keyring/types";
import {
  GENESIS_ACCOUNT,
  ALITH_PRIV_KEY,
  GENESIS_ACCOUNT_PRIVATE_KEY,
  ZERO_ADDRESS,
} from "../util/constants";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createBlockWithExtrinsic } from "../util/substrate-rpc";

const TWENTY_PERCENT = 20;
const TWENTY_PERCENT_STRING = "20.00%";

describeDevMoonbeam("Staking - Parachain Bond - genesis and setParachainBondAccount", (context) => {
  let sudoAccount: KeyringPair;
  before("Setup genesis account for substrate", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
  });
  it("should have right parachain bond in genesis", async function () {
    const parachainBondInfo = await context.polkadotApi.query.parachainStaking.parachainBondInfo();
    expect(parachainBondInfo.toHuman()["account"]).to.equal(ZERO_ADDRESS);
    expect(parachainBondInfo.toHuman()["percent"]).to.equal("30.00%");
  });

  it("should be able set the parachain bond with sudo", async function () {
    // should be able to register the genesis account for reward
    await context.polkadotApi.tx.sudo
      .sudo(context.polkadotApi.tx.parachainStaking.setParachainBondAccount(GENESIS_ACCOUNT))
      .signAndSend(sudoAccount);
    await context.createBlock();
    const parachainBondInfo = await context.polkadotApi.query.parachainStaking.parachainBondInfo();
    expect(parachainBondInfo.toHuman()["account"]).to.equal(GENESIS_ACCOUNT);
    expect(parachainBondInfo.toHuman()["percent"]).to.equal("30.00%");
  });
});

describeDevMoonbeam("Staking - Parachain Bond - no sudo on setParachainBondAccount", (context) => {
  let genesisAccount: KeyringPair, sudoAccount: KeyringPair;

  before("Setup genesis account for substrate", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    genesisAccount = await keyring.addFromUri(GENESIS_ACCOUNT_PRIVATE_KEY, null, "ethereum");
    sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
  });

  it("should NOT be able set the parachain bond if NOT sudo", async function () {
    // should be able to register the genesis account for reward
    try {
      await createBlockWithExtrinsic(
        context,
        genesisAccount,
        context.polkadotApi.tx.authorMapping.setParachainBondAccount(GENESIS_ACCOUNT)
      );
    } catch (e) {
      // NB: This test used to check events for ExtrinsicFailed,
      // but now the api prevents the call from happening
      expect(e.toString().substring(0, 90)).to.eq(
        "TypeError: context.polkadotApi.tx.authorMapping.setParachainBondAccount is not a function"
      );
    }
  });
});

describeDevMoonbeam("Staking - Parachain Bond - setParachainBondReservePercent", (context) => {
  let sudoAccount: KeyringPair;

  before("Setup genesis account for substrate", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
  });

  it("should be able set the parachain bond reserve percent with sudo", async function () {
    // should be able to register the genesis account
    await context.polkadotApi.tx.sudo
      .sudo(context.polkadotApi.tx.parachainStaking.setParachainBondReservePercent(TWENTY_PERCENT))
      .signAndSend(sudoAccount);
    await context.createBlock();
    const parachainBondInfo = await context.polkadotApi.query.parachainStaking.parachainBondInfo();
    expect(parachainBondInfo.toHuman()["account"]).to.equal(ZERO_ADDRESS);
    expect(parachainBondInfo.toHuman()["percent"]).to.equal(TWENTY_PERCENT_STRING);
  });
});

describeDevMoonbeam(
  "Staking - Parachain Bond - no sudo on setParachainBondReservePercent",
  (context) => {
    let genesisAccount: KeyringPair, sudoAccount: KeyringPair;
    before("Setup genesis account for substrate", async () => {
      const keyring = new Keyring({ type: "ethereum" });
      genesisAccount = await keyring.addFromUri(GENESIS_ACCOUNT_PRIVATE_KEY, null, "ethereum");
      sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
    });
    it("should NOT be able set the parachain bond reserve percent without sudo", async function () {
      // should be able to register the genesis account for reward
      try {
        await createBlockWithExtrinsic(
          context,
          genesisAccount,
          context.polkadotApi.tx.authorMapping.setParachainBondReservePercent(TWENTY_PERCENT)
        );
      } catch (e) {
        // NB: This test used to check events for ExtrinsicFailed,
        // but now the api prevents the call from happening
        expect(e.toString().substring(0, 88)).to.eq(
          "TypeError: context.polkadotApi.tx.authorMapping.setParachainBondReservePercent is not a "
        );
      }
    });
  }
);

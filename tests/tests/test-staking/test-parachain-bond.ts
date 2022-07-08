import { expect } from "chai";

import { alith } from "../../util/accounts";
import { ZERO_ADDRESS } from "../../util/constants";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";

const TWENTY_PERCENT = 20;
const TWENTY_PERCENT_STRING = "20.00%";

describeDevMoonbeam("Staking - Parachain Bond", (context) => {
  it("should be initiazed at address zero", async function () {
    const parachainBondInfo = await context.polkadotApi.query.parachainStaking.parachainBondInfo();
    expect(parachainBondInfo.account.toString()).to.equal(ZERO_ADDRESS);
    expect(parachainBondInfo.percent.toNumber()).to.equal(30);
  });

  it("should be changeable to alith address", async function () {
    // should be able to register the genesis account for reward
    await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.parachainStaking.setParachainBondAccount(alith.address)
      )
    );

    const parachainBondInfo = await context.polkadotApi.query.parachainStaking.parachainBondInfo();
    expect(parachainBondInfo.account.toString()).to.equal(alith.address);
    expect(parachainBondInfo.percent.toNumber()).to.equal(30);
  });
});

describeDevMoonbeam("Staking - Parachain Bond - no sudo on setParachainBondAccount", (context) => {
  it("should NOT be able set the parachain bond if NOT sudo", async function () {
    // should be able to register the genesis account for reward
    try {
      await context.createBlock(
        context.polkadotApi.tx.authorMapping.setParachainBondAccount(alith.address)
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
  it("should be able set the parachain bond reserve percent with sudo", async function () {
    // should be able to register the genesis account
    await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.parachainStaking.setParachainBondReservePercent(TWENTY_PERCENT)
      )
    );
    const parachainBondInfo = await context.polkadotApi.query.parachainStaking.parachainBondInfo();
    expect(parachainBondInfo.toHuman()["account"]).to.equal(ZERO_ADDRESS);
    expect(parachainBondInfo.toHuman()["percent"]).to.equal(TWENTY_PERCENT_STRING);
  });
});

describeDevMoonbeam(
  "Staking - Parachain Bond - no sudo on setParachainBondReservePercent",
  (context) => {
    it("should NOT be able set the parachain bond reserve percent without sudo", async function () {
      // should be able to register the genesis account for reward
      try {
        await context.createBlock(
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

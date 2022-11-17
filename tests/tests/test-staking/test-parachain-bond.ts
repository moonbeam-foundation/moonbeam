import { expect } from "chai";

import { alith } from "../../util/accounts";
import { ZERO_ADDRESS } from "../../util/constants";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";

const TWENTY_PERCENT = 20;
const TWENTY_PERCENT_STRING = "20.00%";

describeDevMoonbeam("Staking - Parachain Bond - set bond account", (context) => {
  it("should be initialized at address zero", async function () {
    const parachainBondInfo = await context.polkadotApi.query.parachainStaking.parachainBondInfo();
    expect(parachainBondInfo.account.toString()).to.equal(ZERO_ADDRESS);
    expect(parachainBondInfo.percent.toNumber()).to.equal(30);
  });

  it("should be changeable to alith address", async function () {
    // should be able to register the genesis account for reward
    const { result } = await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.parachainStaking.setParachainBondAccount(alith.address)
      )
    );
    expect(result.successful).to.be.true;

    const parachainBondInfo = await context.polkadotApi.query.parachainStaking.parachainBondInfo();
    expect(parachainBondInfo.account.toString()).to.equal(alith.address);
    expect(parachainBondInfo.percent.toNumber()).to.equal(30);
  });
});

describeDevMoonbeam("Staking - Parachain Bond - no sudo on setParachainBondAccount", (context) => {
  it("should NOT be able set the parachain bond if NOT sudo", async function () {
    // should be able to register the genesis account for reward
    const { result } = await context.createBlock(
      context.polkadotApi.tx.parachainStaking.setParachainBondAccount(alith.address)
    );
    expect(result.successful).to.be.false;
    expect(result.error.name).to.equal("BadOrigin");
  });
});

describeDevMoonbeam("Staking - Parachain Bond - setParachainBondReservePercent", (context) => {
  it("should be able set the parachain bond reserve percent with sudo", async function () {
    // should be able to register the genesis account
    const { result } = await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.parachainStaking.setParachainBondReservePercent(TWENTY_PERCENT)
      )
    );
    expect(result.successful).to.be.true;

    const parachainBondInfo = await context.polkadotApi.query.parachainStaking.parachainBondInfo();
    expect(parachainBondInfo.toHuman()["account"]).to.equal(ZERO_ADDRESS);
    expect(parachainBondInfo.toHuman()["percent"]).to.equal(TWENTY_PERCENT_STRING);
  });
});

describeDevMoonbeam(
  "Staking - Parachain Bond - no sudo on setParachainBondReservePercent",
  (context) => {
    it("should NOT be able set the parachain bond reserve percent without sudo", async function () {
      const { result } = await context.createBlock(
        context.polkadotApi.tx.parachainStaking.setParachainBondReservePercent(TWENTY_PERCENT)
      );
      expect(result.successful).to.be.false;
      expect(result.error.name).to.equal("BadOrigin");
    });
  }
);

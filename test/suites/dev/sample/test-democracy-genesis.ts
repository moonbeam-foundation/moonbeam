import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { describeDevMoonbeam } from "../../util/setup-dev-tests";

describeDevMoonbeam("Democracy - genesis", (context) => {
  it("should have no referendum", async function () {
    expect((await context.polkadotApi.query.democracy.referendumCount()).toNumber()).to.equal(0);
  });
  it("should have no preimages", async function () {
    expect((await context.polkadotApi.query.preimage.preimageFor.entries()).length).to.equal(0);
  });
  it("should have an enactment of 7500", async function () {
    expect(context.polkadotApi.consts.democracy.enactmentPeriod.toNumber()).to.equal(7500);
  });
  it("should have a voting period of 36000", async function () {
    expect(context.polkadotApi.consts.democracy.votingPeriod.toNumber()).to.equal(36000);
  });
  it("should have a launch period of 7200", async function () {
    expect(context.polkadotApi.consts.democracy.launchPeriod.toNumber()).to.equal(7200);
  });
  it("should have a locking period of 7200", async function () {
    expect(context.polkadotApi.consts.democracy.voteLockingPeriod.toNumber()).to.equal(7200);
  });
  it("should have a cooloff period of 50400", async function () {
    expect(context.polkadotApi.consts.democracy.cooloffPeriod.toNumber()).to.equal(50400);
  });
  it("should have instant fast track allowed", async function () {
    expect(context.polkadotApi.consts.democracy.instantAllowed.isTrue).to.be.true;
  });
});

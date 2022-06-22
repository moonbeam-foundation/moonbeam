import "@moonbeam-network/api-augment";

import { u32 } from "@polkadot/types-codec";
import { expect } from "chai";

import { alith } from "../../util/accounts";
import { GLMR, PROPOSAL_AMOUNT } from "../../util/constants";
import { notePreimage } from "../../util/governance";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";

describeDevMoonbeam("Democracy - proposing a vote", (context) => {
  let encodedHash: string;

  before("Create preimage & proposal", async () => {
    // notePreimage
    encodedHash = await notePreimage(
      context,
      context.polkadotApi.tx.parachainStaking.setParachainBondAccount(alith.address),
      alith
    );
    // propose
    await context.createBlock(
      context.polkadotApi.tx.democracy.propose(encodedHash, PROPOSAL_AMOUNT)
    );
  });

  it("should not create a referendum", async function () {
    // referendumCount
    const referendumCount = await context.polkadotApi.query.democracy.referendumCount();
    expect(referendumCount.toBigInt()).to.equal(0n);
  });

  it("should increase the number of proposals to 1", async function () {
    // publicPropCount
    const publicPropCount = await context.polkadotApi.query.democracy.publicPropCount();
    expect(publicPropCount.toBigInt()).to.equal(1n);
  });

  it("should create a proposal", async function () {
    // publicProps
    const publicProps = await context.polkadotApi.query.democracy.publicProps();
    // encodedHash
    expect(publicProps[0][1].toString()).to.equal(encodedHash);
    // prop author
    expect(publicProps[0][2].toString()).to.equal(alith.address);
    // depositOf
  });

  it("should include a deposit of 1000 TOKENs", async function () {
    const depositOf = await context.polkadotApi.query.democracy.depositOf(0);
    expect(depositOf.unwrap()[1].toBigInt()).to.equal(1000n * GLMR);
  });
});

describeDevMoonbeam("Democracy - Seconding a proposal", (context) => {
  let encodedHash: string;
  let launchPeriod: u32;

  before("Setup genesis account for substrate", async () => {
    //launchPeriod
    launchPeriod = await context.polkadotApi.consts.democracy.launchPeriod;

    // notePreimage
    encodedHash = await notePreimage(
      context,
      context.polkadotApi.tx.parachainStaking.setParachainBondAccount(alith.address),
      alith
    );

    // propose & second
    await context.createBlock(
      context.polkadotApi.tx.democracy.propose(encodedHash, PROPOSAL_AMOUNT)
    );
    await context.createBlock(context.polkadotApi.tx.democracy.second(0, 1000));
  });

  it("should succeed", async function () {
    // publicProps
    const publicProps = await context.polkadotApi.query.democracy.publicProps();
    // encodedHash
    expect(publicProps[0][1].toString()).to.equal(encodedHash);
    // prop author
    expect(publicProps[0][2].toString()).to.equal(alith.address);

    // depositOf
    const depositOf = await context.polkadotApi.query.democracy.depositOf(0);
    expect(depositOf.unwrap()[1].toBigInt()).to.equal(1000n * GLMR);
    expect(depositOf.unwrap()[0][1].toString()).to.equal(alith.address);
  });

  it("should have a launch period of 7200", async function () {
    // launchPeriod
    expect(launchPeriod.toBigInt()).to.equal(7200n);
  });
});

describeDevMoonbeam("Democracy - Seconding a proposal", (context) => {
  let encodedHash: string;
  let launchPeriod: u32;

  before("Setup genesis account for substrate", async () => {
    //launchPeriod
    launchPeriod = await context.polkadotApi.consts.democracy.launchPeriod;

    // notePreimage
    encodedHash = await notePreimage(
      context,
      context.polkadotApi.tx.parachainStaking.setParachainBondAccount(alith.address),
      alith
    );

    // propose & second
    await context.createBlock(
      context.polkadotApi.tx.democracy.propose(encodedHash, PROPOSAL_AMOUNT)
    );
    await context.createBlock(context.polkadotApi.tx.democracy.second(0, 1000));
  });

  it("should end-up in a valid referendum", async function () {
    this.timeout(1000000);
    // let Launchperiod elapse to turn the proposal into a referendum
    // launchPeriod minus the 3 blocks that already elapsed
    for (let i = 0; i < Number(launchPeriod) - 3; i++) {
      await context.createBlock();
    }
    // referendumCount
    let referendumCount = await context.polkadotApi.query.democracy.referendumCount();
    expect(referendumCount.toHuman()).to.equal("1");

    // publicPropCount
    const publicPropCount = await context.polkadotApi.query.democracy.publicPropCount();
    expect(publicPropCount.toHuman()).to.equal("1");

    // referendumInfoOf
    const referendumInfoOf = await context.polkadotApi.query.democracy.referendumInfoOf(0);
    expect((referendumInfoOf.toHuman() as any).Ongoing.proposalHash).to.equal(encodedHash);
  });
});

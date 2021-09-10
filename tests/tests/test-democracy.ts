import { Keyring } from "@polkadot/api";
import { KeyringPair } from "@polkadot/keyring/types";
import { expect } from "chai";
import {
  GENESIS_ACCOUNT,
  GENESIS_ACCOUNT_PRIVATE_KEY,
  GLMR,
  ALITH_PRIV_KEY,
  ALITH,
  PROPOSAL_AMOUNT,
  VOTE_AMOUNT,
  ZERO_ADDRESS,
} from "../util/constants";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { notePreimage } from "../util/governance";
import { blake2AsHex } from "@polkadot/util-crypto";
import { createBlockWithExtrinsic } from "../util/substrate-rpc";

describeDevMoonbeam("Democracy - genesis and preimage", (context) => {
  let genesisAccount: KeyringPair;

  before("Setup genesis account for substrate", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    genesisAccount = await keyring.addFromUri(GENESIS_ACCOUNT_PRIVATE_KEY, null, "ethereum");
  });
  it("should check initial state - no referendum", async function () {
    // referendumCount
    const referendumCount = await context.polkadotApi.query.democracy.referendumCount();
    expect(referendumCount.toHuman()).to.equal("0");
  });
  it("should check initial state - 0x0 ParachainBondAccount", async function () {
    // referendumCount
    const parachainBondInfo = await context.polkadotApi.query.parachainStaking.parachainBondInfo();
    expect(parachainBondInfo.toHuman()["account"]).to.equal(ZERO_ADDRESS);
  });
  it("notePreimage", async function () {
    // notePreimage
    const encodedHash = await notePreimage(
      context,
      context.polkadotApi.tx.parachainStaking.setParachainBondAccount(GENESIS_ACCOUNT),
      genesisAccount
    );

    const preimageStatus = await context.polkadotApi.query.democracy.preimages(encodedHash);
    expect((preimageStatus.toHuman() as any).Available.provider).to.equal(GENESIS_ACCOUNT);
    expect((preimageStatus.toHuman() as any).Available.deposit).to.equal("2.2000 mUNIT");
  });
});

describeDevMoonbeam("Democracy - propose", (context) => {
  let genesisAccount: KeyringPair;
  let encodedHash: string;

  before("Setup genesis account for substrate", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    genesisAccount = await keyring.addFromUri(GENESIS_ACCOUNT_PRIVATE_KEY, null, "ethereum");

    // notePreimage
    encodedHash = await notePreimage(
      context,
      context.polkadotApi.tx.parachainStaking.setParachainBondAccount(GENESIS_ACCOUNT),
      genesisAccount
    );
  });
  it("propose", async function () {
    // propose
    await context.polkadotApi.tx.democracy
      .propose(encodedHash, PROPOSAL_AMOUNT)
      .signAndSend(genesisAccount);
    await context.createBlock();

    // referendumCount
    const referendumCount = await context.polkadotApi.query.democracy.referendumCount();
    expect(referendumCount.toHuman()).to.equal("0");

    // publicPropCount
    const publicPropCount = await context.polkadotApi.query.democracy.publicPropCount();
    expect(publicPropCount.toHuman()).to.equal("1");

    // publicProps
    const publicProps = await context.polkadotApi.query.democracy.publicProps();
    // encodedHash
    expect((publicProps.toHuman() as any)[0][1]).to.equal(encodedHash);
    // prop author
    expect((publicProps.toHuman() as any)[0][2]).to.equal(GENESIS_ACCOUNT);
    // depositOf
    const depositOf = await context.polkadotApi.query.democracy.depositOf(0);
    expect((depositOf.toHuman() as any)[1]).to.equal("1.0000 kUNIT");
  });
});

describeDevMoonbeam("Democracy - second proposal", (context) => {
  let genesisAccount: KeyringPair, alith: KeyringPair;
  let encodedHash: string;
  let launchPeriod;

  before("Setup genesis account for substrate", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    genesisAccount = await keyring.addFromUri(GENESIS_ACCOUNT_PRIVATE_KEY, null, "ethereum");
    alith = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");

    //launchPeriod
    launchPeriod = await context.polkadotApi.consts.democracy.launchPeriod;

    // notePreimage
    encodedHash = await notePreimage(
      context,
      context.polkadotApi.tx.parachainStaking.setParachainBondAccount(GENESIS_ACCOUNT),
      genesisAccount
    );

    // propose
    await context.polkadotApi.tx.democracy
      .propose(encodedHash, PROPOSAL_AMOUNT)
      .signAndSend(genesisAccount);
    await context.createBlock();

    // second
    await context.polkadotApi.tx.democracy.second(0, 1000).signAndSend(alith);
    await context.createBlock();
  });
  it("second proposal", async function () {
    // publicProps
    const publicProps = await context.polkadotApi.query.democracy.publicProps();
    // encodedHash
    expect((publicProps.toHuman() as any)[0][1]).to.equal(encodedHash);
    // prop author
    expect((publicProps.toHuman() as any)[0][2]).to.equal(GENESIS_ACCOUNT);

    // depositOf
    const depositOf = await context.polkadotApi.query.democracy.depositOf(0);
    expect((depositOf.toHuman() as any)[1]).to.equal("1.0000 kUNIT");
    expect((depositOf.toHuman() as any)[0][1]).to.equal(ALITH);
  });
  it("check launch period", async function () {
    // launchPeriod
    expect(launchPeriod.toHuman()).to.equal("7,200");
  });
  it("check referendum is up", async function () {
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

describeDevMoonbeam("Democracy - vote on referendum", (context) => {
  let genesisAccount: KeyringPair, alith: KeyringPair;
  let encodedHash: string;
  let enactmentPeriod, votingPeriod;

  before("Setup genesis account for substrate", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    genesisAccount = await keyring.addFromUri(GENESIS_ACCOUNT_PRIVATE_KEY, null, "ethereum");
    alith = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");

    // enactmentPeriod
    enactmentPeriod = await context.polkadotApi.consts.democracy.enactmentPeriod;
    // votingPeriod
    votingPeriod = await context.polkadotApi.consts.democracy.votingPeriod;

    // notePreimage
    encodedHash = await notePreimage(
      context,
      context.polkadotApi.tx.parachainStaking.setParachainBondAccount(GENESIS_ACCOUNT),
      genesisAccount
    );
    // propose
    await context.polkadotApi.tx.democracy
      .propose(encodedHash, PROPOSAL_AMOUNT)
      .signAndSend(genesisAccount);
    await context.createBlock();
    // second
    await context.polkadotApi.tx.democracy.second(0, 1000).signAndSend(alith);
    await context.createBlock();
  });
  it("check enactment period", async function () {
    // enactmentPeriod
    expect(enactmentPeriod.toHuman()).to.equal("7,200");
  });
  it("check voting Period", async function () {
    // votingPeriod
    expect(votingPeriod.toHuman()).to.equal("36,000");
  });
  it("vote", async function () {
    this.timeout(2000000);
    // let Launchperiod elapse to turn the proposal into a referendum
    // launchPeriod minus the 3 blocks that already elapsed
    for (let i = 0; i < 7200 - 3; i++) {
      await context.createBlock();
    }
    // vote
    await context.polkadotApi.tx.democracy
      .vote(0, {
        Standard: { balance: VOTE_AMOUNT, vote: { aye: true, conviction: 1 } },
      })
      .signAndSend(alith);
    await context.createBlock();

    // referendumInfoOf
    const referendumInfoOf = await context.polkadotApi.query.democracy.referendumInfoOf(0);
    expect((referendumInfoOf.toHuman() as any).Ongoing.proposalHash).to.equal(encodedHash);
    expect((referendumInfoOf.toHuman() as any).Ongoing.tally.ayes).to.equal("10.0000 UNIT");
    expect((referendumInfoOf.toHuman() as any).Ongoing.tally.turnout).to.equal("10.0000 UNIT");

    // let votePeriod + enactmentPeriod elapse to turn the proposal into a referendum
    for (let i = 0; i < Number(votingPeriod) + Number(enactmentPeriod); i++) {
      await context.createBlock();
    }
    let parachainBondInfo = await context.polkadotApi.query.parachainStaking.parachainBondInfo();
    expect(parachainBondInfo.toHuman()["account"]).to.equal(GENESIS_ACCOUNT);
  });
});

// When forgetting to call notePreimage, all following steps should work as intended
// until the end where the proposal is never enacted

describeDevMoonbeam("Democracy - forget notePreimage", (context) => {
  let genesisAccount: KeyringPair, alith: KeyringPair;
  let encodedHash: string;
  let enactmentPeriod, votingPeriod;

  before("Setup genesis account for substrate", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    genesisAccount = await keyring.addFromUri(GENESIS_ACCOUNT_PRIVATE_KEY, null, "ethereum");
    alith = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");

    // notePreimage
    // compute proposal hash but don't submit it
    const encodedProposal =
      context.polkadotApi.tx.parachainStaking
        .setParachainBondAccount(GENESIS_ACCOUNT)
        .method.toHex() || "";
    encodedHash = blake2AsHex(encodedProposal);
  });
  it("vote", async function () {
    this.timeout(200000);

    // propose
    const { events: eventsPropose } = await createBlockWithExtrinsic(
      context,
      genesisAccount,
      context.polkadotApi.tx.democracy.propose(encodedHash, PROPOSAL_AMOUNT)
    );
    expect(eventsPropose[5].toHuman().method).to.eq("ExtrinsicSuccess");
    await context.createBlock();
    // second
    const { events: eventsSecond } = await createBlockWithExtrinsic(
      context,
      alith,
      context.polkadotApi.tx.democracy.second(0, 1000)
    );
    expect(eventsSecond[2].toHuman().method).to.eq("ExtrinsicSuccess");
    // let Launchperiod elapse to turn the proposal into a referendum
    // launchPeriod minus the 3 blocks that already elapsed
    for (let i = 0; i < 7200; i++) {
      await context.createBlock();
    }
    // referendumCount
    let referendumCount = await context.polkadotApi.query.democracy.referendumCount();
    expect(referendumCount.toHuman()).to.equal("1");

    // vote
    await context.createBlock();
    const { events: eventsVote } = await createBlockWithExtrinsic(
      context,
      alith,
      context.polkadotApi.tx.democracy.vote(0, {
        Standard: { balance: VOTE_AMOUNT, vote: { aye: true, conviction: 1 } },
      })
    );
    expect(eventsVote[1].toHuman().method).to.eq("ExtrinsicSuccess");

    // referendumInfoOf
    const referendumInfoOf = await context.polkadotApi.query.democracy.referendumInfoOf(0);
    expect((referendumInfoOf.toHuman() as any).Ongoing.proposalHash).to.equal(encodedHash);
    expect((referendumInfoOf.toHuman() as any).Ongoing.tally.ayes).to.equal("10.0000 UNIT");
    expect((referendumInfoOf.toHuman() as any).Ongoing.tally.turnout).to.equal("10.0000 UNIT");

    // let votePeriod + enactmentPeriod elapse to turn the proposal into a referendum
    for (let i = 0; i < Number(votingPeriod) + Number(enactmentPeriod); i++) {
      await context.createBlock();
    }
    // the enactement should fail
    let parachainBondInfo = await context.polkadotApi.query.parachainStaking.parachainBondInfo();
    expect(parachainBondInfo.toHuman()["account"]).to.equal(ZERO_ADDRESS);
  });
});

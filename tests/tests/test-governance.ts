import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { blake2AsHex } from "@polkadot/util-crypto";
import Keyring from "@polkadot/keyring";
import {
  ALITH_PRIVATE_KEY,
  GENESIS_ACCOUNT,
  PROPOSAL_AMOUNT,
  VOTE_AMOUNT,
  ZERO_ADDRESS,
} from "../util/constants";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { execFromTwoThirdsOfCouncil, execFromAllMembersOfTechCommittee } from "../util/governance";
import { BN } from "@polkadot/util";
import { KeyringPair } from "@substrate/txwrapper-core";
import { createBlockWithExtrinsic } from "../util/substrate-rpc";

const keyring = new Keyring({ type: "ethereum" });

let alith;

describeDevMoonbeam("Governance - Democracy and Council Collectve", (context) => {
  before("Create accounts", async () => {
    alith = await keyring.addFromUri(ALITH_PRIVATE_KEY, null, "ethereum");
  });

  it("should be able to submit a proposal", async function () {
    // Alith submit a proposal
    let proposalHash = "0xf3d039875302d49d52fb1af6877a2c46bc55b004afb8130f94dd9d0489ca3185";
    await context.polkadotApi.tx.democracy
      .propose(proposalHash, PROPOSAL_AMOUNT)
      .signAndSend(alith);
    await context.createBlock();

    // Verify that Alith proposal is registered
    const publicPropCount = await context.polkadotApi.query.democracy.publicPropCount();
    expect(publicPropCount.toHuman()).to.equal("1");
  });

  it("should be able to fast track a referundum with councilCollective pallet", async function () {
    // Verify that no referundum is triggered
    expect((await context.polkadotApi.query.democracy.referendumCount()).toHuman()).to.equal("0");

    const proposalHash = "0xf3d039875302d49d52fb1af6877a2c46bc55b004afb8130f94dd9d0489ca3185";
    await execFromTwoThirdsOfCouncil(
      context,
      context.polkadotApi.tx.democracy.externalProposeMajority(proposalHash)
    );
    await execFromAllMembersOfTechCommittee(
      context,
      context.polkadotApi.tx.democracy.fastTrack(proposalHash, 5, 0)
    );

    // Verify that one referundum is triggered
    let referendumCount = await context.polkadotApi.query.democracy.referendumCount();
    expect(referendumCount.toHuman()).to.equal("1");
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
    genesisAccount = await keyring.addFromUri(ALITH_PRIVATE_KEY, null, "ethereum");
    alith = await keyring.addFromUri(ALITH_PRIVATE_KEY, null, "ethereum");
    // notePreimage
    // compute proposal hash but don't submit it
    const encodedProposal =
      context.polkadotApi.tx.parachainStaking
        .setParachainBondAccount(GENESIS_ACCOUNT)
        .method.toHex() || "";
    encodedHash = blake2AsHex(encodedProposal);

    // enactmentPeriod
    enactmentPeriod = await context.polkadotApi.consts.democracy.enactmentPeriod;
    // votingPeriod
    votingPeriod = await context.polkadotApi.consts.democracy.votingPeriod;
  });

  it("vote", async function () {
    this.timeout(200000);

    // propose
    const { events: eventsPropose } = await createBlockWithExtrinsic(
      context,
      genesisAccount,
      context.polkadotApi.tx.democracy.propose(encodedHash, PROPOSAL_AMOUNT)
    );
    expect(eventsPropose.find((e) => e.method.toString() == "Proposed")).to.not.be.empty;
    expect(eventsPropose.find((e) => e.method.toString() == "ExtrinsicSuccess")).to.not.be.empty;
    await context.createBlock();
    // second
    const { events: eventsSecond } = await createBlockWithExtrinsic(
      context,
      alith,
      context.polkadotApi.tx.democracy.second(0, 1000)
    );
    expect(eventsSecond[2].toHuman().method).to.eq("Seconded");
    expect(eventsSecond[5].toHuman().method).to.eq("ExtrinsicSuccess");
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

    expect(eventsVote[1].toHuman().method).to.eq("Voted");
    expect(eventsVote[4].toHuman().method).to.eq("ExtrinsicSuccess");

    // referendumInfoOf
    const referendumInfoOf = await context.polkadotApi.query.democracy.referendumInfoOf(0);
    expect(referendumInfoOf.unwrap().asOngoing.proposalHash.toHex()).to.equal(encodedHash);
    expect(referendumInfoOf.unwrap().asOngoing.delay.toNumber()).to.equal(
      enactmentPeriod.toNumber()
    );
    expect(referendumInfoOf.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(
      10_000_000_000_000_000_000n
    );
    expect(referendumInfoOf.unwrap().asOngoing.tally.turnout.toBigInt()).to.equal(
      10_000_000_000_000_000_000n
    );

    const referendumHex = referendumInfoOf.toHex();

    // Instead of waiting votePeriod + enactmentPeriod (which is very long) we hack
    // the referendum to be shorter
    const blockNumber = (await context.polkadotApi.rpc.chain.getHeader()).number;

    // new end block to
    const newEndBlock = context.polkadotApi.registry.createType(
      "u32",
      blockNumber.toBn().add(new BN(2))
    );

    // Set 0 block delay
    const delay = context.polkadotApi.registry.createType(
      "u32",
      referendumInfoOf.unwrap().asOngoing.delay.sub(enactmentPeriod)
    );

    // taking same referendum with different end & delay\
    const modReferendum = `0x00${newEndBlock.toHex(true).slice(2)}${referendumHex.slice(
      12,
      78
    )}${delay.toHex(true).slice(2)}${referendumHex.slice(86)}`;

    // Changing storage for the referendum using sudo
    await context.polkadotApi.tx.sudo
      .sudo(
        context.polkadotApi.tx.system.setStorage([
          [context.polkadotApi.query.democracy.referendumInfoOf.key(0).toString(), modReferendum],
        ])
      )
      .signAndSend(alith);
    await context.createBlock();

    // Waiting extra blocks for the vote to finish
    for (let i = 0; i < 2; i++) {
      await context.createBlock();
    }
    // the enactement should fail
    let parachainBondInfo =
      (await context.polkadotApi.query.parachainStaking.parachainBondInfo()) as any;
    const referendumDone = await context.polkadotApi.query.democracy.referendumInfoOf(0);
    expect(referendumDone.unwrap().isFinished).to.be.true;
    expect(referendumDone.unwrap().asFinished.approved.isTrue).to.be.true;
    expect(parachainBondInfo.account.toString()).to.equal(ZERO_ADDRESS);
  });
});

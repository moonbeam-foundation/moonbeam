import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { alith } from "../../util/accounts";
import { GLMR, VOTE_AMOUNT, ZERO_ADDRESS } from "../../util/constants";
import { instantFastTrack, notePreimage } from "../../util/governance";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";

describeDevMoonbeam("Democracy - Referendum", (context) => {
  let encodedHash: string;

  before("Setup genesis account for substrate", async () => {
    // notePreimage
    encodedHash = await notePreimage(
      context,
      context.polkadotApi.tx.parachainStaking.setParachainBondAccount(alith.address),
      alith
    );
    await instantFastTrack(context, encodedHash);
  });

  it("should succeed with enough votes", async function () {
    // vote
    await context.createBlock(
      context.polkadotApi.tx.democracy.vote(0, {
        Standard: { balance: VOTE_AMOUNT, vote: { aye: true, conviction: 1 } },
      })
    );

    // referendumInfoOf
    const referendumInfoOf = (
      await context.polkadotApi.query.democracy.referendumInfoOf(0)
    ).unwrap();
    const onGoing = referendumInfoOf.asOngoing;

    expect(onGoing.proposalHash.toHex()).to.equal(encodedHash);
    expect(onGoing.tally.ayes.toBigInt()).to.equal(10n * GLMR);
    expect(onGoing.tally.turnout.toBigInt()).to.equal(10n * GLMR);

    const blockNumber = (await context.polkadotApi.rpc.chain.getHeader()).number.toNumber();
    for (let i = 0; i < onGoing.end.toNumber() - blockNumber; i++) {
      await context.createBlock();
    }

    const finishedReferendum = (
      await context.polkadotApi.query.democracy.referendumInfoOf(0)
    ).unwrap();

    expect(finishedReferendum.isFinished).to.be.true;
    expect(finishedReferendum.asFinished.approved.isTrue).to.be.true;

    let parachainBondInfo = await context.polkadotApi.query.parachainStaking.parachainBondInfo();
    expect(parachainBondInfo.account.toString()).to.equal(alith.address);
  });
});

describeDevMoonbeam("Democracy - Referendum", (context) => {
  let encodedHash: string;

  before("Setup genesis account for substrate", async () => {
    // notePreimage
    encodedHash = await notePreimage(
      context,
      context.polkadotApi.tx.parachainStaking.setParachainBondAccount(alith.address),
      alith
    );
    await instantFastTrack(context, encodedHash);
  });

  it("should fail with enough no votes", async function () {
    // vote
    await context.createBlock(
      context.polkadotApi.tx.democracy.vote(0, {
        Standard: { balance: VOTE_AMOUNT, vote: { aye: false, conviction: 1 } },
      })
    );

    // referendumInfoOf
    const referendumInfoOf = (
      await context.polkadotApi.query.democracy.referendumInfoOf(0)
    ).unwrap();
    const onGoing = referendumInfoOf.asOngoing;

    expect(onGoing.proposalHash.toHex()).to.equal(encodedHash);
    expect(onGoing.tally.nays.toBigInt()).to.equal(10n * GLMR);
    expect(onGoing.tally.turnout.toBigInt()).to.equal(10n * GLMR);

    const blockNumber = (await context.polkadotApi.rpc.chain.getHeader()).number.toNumber();
    for (let i = 0; i < onGoing.end.toNumber() - blockNumber; i++) {
      await context.createBlock();
    }
    const finishedReferendum = (
      await context.polkadotApi.query.democracy.referendumInfoOf(0)
    ).unwrap();

    expect(finishedReferendum.isFinished).to.be.true;
    expect(finishedReferendum.asFinished.approved.isFalse).to.be.true;

    let parachainBondInfo = await context.polkadotApi.query.parachainStaking.parachainBondInfo();
    expect(parachainBondInfo.account.toString()).to.equal(ZERO_ADDRESS);
  });
});

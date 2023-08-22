import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { alith, generateKeyringPair } from "../../util/accounts";
import { GLMR, VOTE_AMOUNT, ZERO_ADDRESS } from "../../util/constants";
import { instantFastTrack, notePreimage } from "../../util/governance";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { expectOk } from "../../util/expect";

describeDevMoonbeam("Democracy - Referendum", (context) => {
  let encodedHash: string;

  before("Setup referendum", async () => {
    // notePreimage
    encodedHash = await instantFastTrack(
      context,
      context.polkadotApi.tx.parachainStaking.setParachainBondAccount(alith.address)
    );
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
    ).unwrap() as any;
    const onGoing = referendumInfoOf.asOngoing;

    expect(onGoing.proposal.asLookup.hash_.toHex()).to.equal(encodedHash);
    expect(onGoing.tally.ayes.toBigInt()).to.equal(10n * GLMR);
    expect(onGoing.tally.turnout.toBigInt()).to.equal(10n * GLMR);

    const blockNumber = (await context.polkadotApi.rpc.chain.getHeader()).number.toNumber();
    for (let i = 0; i < onGoing.end.toNumber() - blockNumber + 1; i++) {
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

  before("Setup a referendum", async () => {
    // notePreimage
    encodedHash = await instantFastTrack(
      context,
      context.polkadotApi.tx.system.remark("Just a simple vote")
    );
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
    ).unwrap() as any;
    const onGoing = referendumInfoOf.asOngoing;

    expect(onGoing.proposal.asLookup.hash_.toHex()).to.equal(encodedHash);
    expect(onGoing.tally.nays.toBigInt()).to.equal(10n * GLMR);
    expect(onGoing.tally.turnout.toBigInt()).to.equal(10n * GLMR);

    const blockNumber = (await context.polkadotApi.rpc.chain.getHeader()).number.toNumber();
    for (let i = 0; i < onGoing.end.toNumber() - blockNumber + 1; i++) {
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

describeDevMoonbeam("Democracy - Referendum", (context) => {
  let encodedHash: string;
  const randomAccount = generateKeyringPair();

  before("Setup a vote & random account & delegation", async () => {
    // notePreimage
    encodedHash = await instantFastTrack(
      context,
      context.polkadotApi.tx.system.remark("Just a simple vote"),
      { votingPeriod: 10, delayPeriod: 1 }
    );
    await context.createBlock(
      context.polkadotApi.tx.balances.transfer(randomAccount.address, 100n * GLMR)
    );
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegate(alith.address, 90n * GLMR, 0, 0)
        .signAsync(randomAccount)
    );
  });

  it("should be votable while staked", async function () {
    const { result } = await context.createBlock(
      context.polkadotApi.tx.democracy
        .vote(0, {
          Standard: { balance: 90n * GLMR, vote: { aye: false, conviction: 1 } },
        })
        .signAsync(randomAccount)
    );

    expect(result.successful).to.be.true;

    // ensure we have both locks
    const locks = await context.polkadotApi.query.balances.locks(randomAccount.address);
    expect(locks.length).to.be.equal(2, "Failed to incur two locks");
    expect(locks[0].amount.toBigInt()).to.be.equal(90n * GLMR);
    expect(locks[0].id.toHuman().toString()).to.be.equal("stkngdel");
    expect(locks[1].amount.toBigInt()).to.be.equal(90n * GLMR);
    expect(locks[1].id.toHuman().toString()).to.be.equal("democrac");
  });
});

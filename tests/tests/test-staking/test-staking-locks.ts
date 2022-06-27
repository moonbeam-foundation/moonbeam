import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { alith, baltathar, generateKeyingPair } from "../../util/accounts";
import {
  execCouncilProposal,
  execTechnicalCommitteeProposal,
  notePreimage,
} from "../../util/governance";
import { GLMR, MIN_GLMR_STAKING } from "../../util/constants";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { jumpToRound, shortcutRounds } from "../../util/block";

const DELEGATE_AMOUNT = 100n * GLMR;
describeDevMoonbeam("Staking - Locks", (context) => {
  const randomAccount = generateKeyingPair();

  before("Setup account balance", async function () {
    await context.createBlock(
      context.polkadotApi.tx.balances.transfer(randomAccount.address, DELEGATE_AMOUNT + 1n * GLMR)
    );
  });

  it("should be set when staking", async function () {
    const { result } = await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegate(alith.address, DELEGATE_AMOUNT, 10, 10)
        .signAsync(randomAccount)
    );
    const locks = await context.polkadotApi.query.balances.locks(randomAccount.address);
    expect(result.successful).to.be.true;
    expect(locks.length).to.be.equal(1, "Missing lock");
    expect(locks[0].amount.toBigInt()).to.be.equal(DELEGATE_AMOUNT);
    expect(locks[0].id.toHuman().toString()).to.be.equal("DelStake");
  });
});

describeDevMoonbeam("Staking - Locks", (context) => {
  const randomAccount = generateKeyingPair();

  before("Setup account balance & staking", async function () {
    await context.createBlock(
      context.polkadotApi.tx.balances.transfer(randomAccount.address, DELEGATE_AMOUNT + 1n * GLMR)
    );
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegate(alith.address, DELEGATE_AMOUNT, 10, 10)
        .signAsync(randomAccount)
    );
  });

  it("should not be reusable for staking", async function () {
    const { result } = await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegate(baltathar.address, DELEGATE_AMOUNT, 10, 10)
        .signAsync(randomAccount)
    );
    expect(result.error.name.toString()).to.be.equal("InsufficientBalance");
  });
});

describeDevMoonbeam("Staking - Locks", (context) => {
  const randomAccount = generateKeyingPair();

  before("Setup account balance & staking", async function () {
    await context.createBlock(
      context.polkadotApi.tx.balances.transfer(randomAccount.address, DELEGATE_AMOUNT + 1n * GLMR)
    );
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegate(alith.address, DELEGATE_AMOUNT, 10, 10)
        .signAsync(randomAccount)
    );

    const proposalHash = await notePreimage(
      context,
      context.polkadotApi.tx.parachainStaking.setParachainBondAccount(alith.address),
      alith
    );
    await execCouncilProposal(
      context,
      context.polkadotApi.tx.democracy.externalProposeMajority(proposalHash)
    );
    await execTechnicalCommitteeProposal(
      context,
      context.polkadotApi.tx.democracy.fastTrack(proposalHash, 100, 1)
    );
  });

  it("should be usable for democracy vote", async function () {
    const { result } = await context.createBlock(
      context.polkadotApi.tx.democracy
        .vote(0, {
          Standard: { balance: DELEGATE_AMOUNT, vote: { aye: true, conviction: 1 } },
        })
        .signAsync(randomAccount)
    );
    expect(result.successful).to.be.true;
    expect(result.events.find(({ event: { method } }) => method == "Voted")).to.not.be.undefined;
  });
});

describeDevMoonbeam("Staking - Locks", (context) => {
  const randomAccount = generateKeyingPair();

  before("Setup account balance", async function () {
    await context.createBlock(
      context.polkadotApi.tx.balances.transfer(randomAccount.address, DELEGATE_AMOUNT + 1n * GLMR)
    );
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegate(alith.address, DELEGATE_AMOUNT, 10, 10)
        .signAsync(randomAccount)
    );
  });

  it("should be unlocked only after executing revoke delegation", async function () {
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .scheduleRevokeDelegation(alith.address)
        .signAsync(randomAccount)
    );

    // Additional check
    const locks = await context.polkadotApi.query.balances.locks(randomAccount.address);
    expect(locks[0].id.toHuman().toString()).to.be.equal("DelStake");

    // Fast track 2 next rounds
    const rewardDelay = context.polkadotApi.consts.parachainStaking.rewardPaymentDelay;
    await shortcutRounds(context, rewardDelay.toNumber());

    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .executeDelegationRequest(randomAccount.address, alith.address)
        .signAsync(randomAccount)
    );

    const newLocks = await context.polkadotApi.query.balances.locks(randomAccount.address);
    expect(newLocks.length).to.be.equal(0, "Lock should have been removed after executing revoke");
  });
});

describeDevMoonbeam("Staking - Locks", (context) => {
  const randomAccount = generateKeyingPair();

  before("Setup candidate & delegations", async function () {
    await context.createBlock([
      context.polkadotApi.tx.balances.transfer(
        randomAccount.address,
        MIN_GLMR_STAKING * 2n + 1n * GLMR
      ),
      context.polkadotApi.tx.parachainStaking
        .joinCandidates(MIN_GLMR_STAKING, 1)
        .signAsync(baltathar),
    ]);

    let nonce = await context.web3.eth.getTransactionCount(randomAccount.address);
    await context.createBlock([
      context.polkadotApi.tx.parachainStaking
        .delegate(alith.address, DELEGATE_AMOUNT, 10, 10)
        .signAsync(randomAccount, { nonce: nonce++ }),
      context.polkadotApi.tx.parachainStaking
        .delegate(baltathar.address, DELEGATE_AMOUNT, 10, 10)
        .signAsync(randomAccount, { nonce: nonce++ }),
    ]);
  });

  it("should be unlocked only after executing the last revoke delegation", async function () {
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .scheduleRevokeDelegation(alith.address)
        .signAsync(randomAccount)
    );

    // Additional check
    const locks = await context.polkadotApi.query.balances.locks(randomAccount.address);
    expect(locks[0].id.toHuman().toString()).to.be.equal("DelStake");

    // Fast track 2 next rounds
    const rewardDelay = context.polkadotApi.consts.parachainStaking.rewardPaymentDelay;
    await shortcutRounds(context, rewardDelay.toNumber());

    const { result } = await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .executeDelegationRequest(randomAccount.address, alith.address)
        .signAsync(randomAccount)
    );
    expect(result.successful).to.be.true;

    // Additional check we have still have 1 delegation
    const delegatorState = await context.polkadotApi.query.parachainStaking.delegatorState(
      randomAccount.address
    );
    expect(delegatorState.unwrap().delegations.length).to.be.equal(1, "Missing delegation");
    // Only 1 over the 2 delegations has been revoked
    const newLocks = await context.polkadotApi.query.balances.locks(randomAccount.address);
    expect(newLocks.length).to.be.equal(1, "Missing lock");
    expect(newLocks[0].amount.toBigInt()).to.be.equal(DELEGATE_AMOUNT);
    expect(newLocks[0].id.toHuman().toString()).to.be.equal("DelStake");
  });
});

describeDevMoonbeam("Staking - Locks", (context) => {
  const randomAccount = generateKeyingPair();

  before("Setup candidate & delegations", async function () {
    await context.createBlock([
      context.polkadotApi.tx.balances.transfer(
        randomAccount.address,
        MIN_GLMR_STAKING * 2n + 1n * GLMR
      ),
      context.polkadotApi.tx.parachainStaking
        .joinCandidates(MIN_GLMR_STAKING, 1)
        .signAsync(baltathar),
    ]);

    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegate(alith.address, DELEGATE_AMOUNT, 10, 10)
        .signAsync(randomAccount)
    );
  });

  it("should not be created for additional delegations", async function () {
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegate(baltathar.address, DELEGATE_AMOUNT, 10, 10)
        .signAsync(randomAccount)
    );

    // Additional check
    const delegatorState = await context.polkadotApi.query.parachainStaking.delegatorState(
      randomAccount.address
    );
    expect(delegatorState.unwrap().delegations.length).to.be.equal(2, "Missing delegation");

    const locks = await context.polkadotApi.query.balances.locks(randomAccount.address);
    expect(locks.length).to.be.equal(
      1,
      `Unexpected number of locks: ${locks.map((l) => l.id.toHuman().toString()).join(` - `)}`
    );
    expect(locks[0].amount.toBigInt(), `Unexpected amount for lock`).to.be.equal(DELEGATE_AMOUNT);
    expect(locks[0].id.toHuman().toString()).to.be.equal("DelStake");
  });
});

describeDevMoonbeam("Staking - Locks", (context) => {
  it("should be created when joining candidates", async function () {
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .joinCandidates(MIN_GLMR_STAKING, 1)
        .signAsync(baltathar)
    );

    const locks = await context.polkadotApi.query.balances.locks(baltathar.address);
    expect(locks.length).to.be.equal(
      1,
      `Unexpected number of locks: ${locks.map((l) => l.id.toHuman().toString()).join(` - `)}`
    );
    expect(locks[0].amount.toBigInt()).to.be.equal(MIN_GLMR_STAKING);
    expect(locks[0].id.toHuman().toString()).to.be.equal("ColStake");
  });
});

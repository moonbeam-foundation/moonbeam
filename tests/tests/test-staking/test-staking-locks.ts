import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { alith, baltathar, generateKeyingPair } from "../../util/accounts";
import {
  execCouncilProposal,
  execTechnicalCommitteeProposal,
  notePreimage,
} from "../../util/governance";
import { GLMR, MIN_GLMR_STAKING } from "../../util/constants";
import { describeDevMoonbeam, DevTestContext } from "../../util/setup-dev-tests";
import { shortcutRounds } from "../../util/block";
import { KeyringPair } from "@polkadot/keyring/types";

const DELEGATE_AMOUNT = 100n * GLMR;
describeDevMoonbeam("Staking - Locks", (context) => {
  const randomAccount = generateKeyingPair();

  before("Setup account balance", async function () {
    await context.createBlock(
      context.polkadotApi.tx.balances.transfer(randomAccount.address, DELEGATE_AMOUNT + 1n * GLMR)
    );
  });

  it('should set "stkngdel" when delegating', async function () {
    const { result } = await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegate(alith.address, DELEGATE_AMOUNT, 10, 10)
        .signAsync(randomAccount)
    );
    const locks = await context.polkadotApi.query.balances.locks(randomAccount.address);
    expect(result.successful).to.be.true;
    expect(locks.length).to.be.equal(1, "Missing lock");
    expect(locks[0].amount.toBigInt()).to.be.equal(DELEGATE_AMOUNT);
    expect(locks[0].id.toHuman().toString()).to.be.equal("stkngdel");
  });
});

describeDevMoonbeam("Staking - Locks", (context) => {
  const randomAccount = generateKeyingPair();

  before("Setup account balance & delegation", async function () {
    await context.createBlock(
      context.polkadotApi.tx.balances.transfer(randomAccount.address, DELEGATE_AMOUNT + 1n * GLMR)
    );
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegate(alith.address, DELEGATE_AMOUNT, 10, 10)
        .signAsync(randomAccount)
    );
  });

  it("should not be reusable for delegation", async function () {
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

  before("Setup account balance & delegation", async function () {
    await context.createBlock(
      context.polkadotApi.tx.balances.transfer(randomAccount.address, DELEGATE_AMOUNT + 1n * GLMR)
    );
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegate(alith.address, DELEGATE_AMOUNT, 10, 10)
        .signAsync(randomAccount)
    );
  });

  it("should not be reusable for candidate", async function () {
    const { result } = await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .joinCandidates(MIN_GLMR_STAKING, 1)
        .signAsync(randomAccount)
    );
    expect(result.error.name.toString()).to.be.equal("DelegatorExists");
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

  it("should stay locked after requesting a delegation revoke", async function () {
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .scheduleRevokeDelegation(alith.address)
        .signAsync(randomAccount)
    );

    // Additional check
    const locks = await context.polkadotApi.query.balances.locks(randomAccount.address);
    expect(locks[0].id.toHuman().toString()).to.be.equal("stkngdel");
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

    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .scheduleRevokeDelegation(alith.address)
        .signAsync(randomAccount)
    );

    // Fast track 2 next rounds
    const rewardDelay = context.polkadotApi.consts.parachainStaking.rewardPaymentDelay;
    await shortcutRounds(context, rewardDelay.toNumber());

    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .executeDelegationRequest(randomAccount.address, alith.address)
        .signAsync(randomAccount)
    );
  });

  it("should be removed only after executing the last revoke delegation", async function () {
    // Additional check we have still have 1 delegation
    const delegatorState = await context.polkadotApi.query.parachainStaking.delegatorState(
      randomAccount.address
    );
    expect(delegatorState.unwrap().delegations.length).to.be.equal(1, "Missing delegation");
    // Only 1 over the 2 delegations has been revoked
    const locks = await context.polkadotApi.query.balances.locks(randomAccount.address);
    expect(locks.length).to.be.equal(1, "Missing lock");
    expect(locks[0].amount.toBigInt()).to.be.equal(DELEGATE_AMOUNT);
    expect(locks[0].id.toHuman().toString()).to.be.equal("stkngdel");
  });
});

describeDevMoonbeam("Staking - Locks", (context) => {
  const randomAccount = generateKeyingPair();
  let randomCandidates: KeyringPair[];
  let maxDelegationsPerDelegator: bigint;

  before("Setup candidate & delegations", async function () {
    this.timeout(12000);
    maxDelegationsPerDelegator =
      context.polkadotApi.consts.parachainStaking.maxDelegationsPerDelegator.toBigInt();
    randomCandidates = new Array(Number(maxDelegationsPerDelegator))
      .fill(0)
      .map(() => generateKeyingPair());

    let alithNonce = await context.web3.eth.getTransactionCount(alith.address);
    await context.createBlock([
      context.polkadotApi.tx.balances
        .transfer(randomAccount.address, (DELEGATE_AMOUNT + GLMR) * maxDelegationsPerDelegator)
        .signAsync(alith, { nonce: alithNonce++ }),
      ...randomCandidates.map((randomCandidate) =>
        context.polkadotApi.tx.balances
          .transfer(randomCandidate.address, MIN_GLMR_STAKING + 1n * GLMR)
          .signAsync(alith, { nonce: alithNonce++ })
      ),
    ]);

    await context.createBlock(
      randomCandidates.map((randomCandidate) =>
        context.polkadotApi.tx.parachainStaking
          .joinCandidates(MIN_GLMR_STAKING, maxDelegationsPerDelegator)
          .signAsync(randomCandidate)
      )
    );

    const candidates = await context.polkadotApi.query.parachainStaking.candidateInfo.entries();
    expect(candidates.length).to.be.equal(
      Number(maxDelegationsPerDelegator) + 1,
      "Missing candidates"
    );

    let nonce = await context.web3.eth.getTransactionCount(randomAccount.address);
    await context.createBlock(
      randomCandidates.map((randomCandidate) =>
        context.polkadotApi.tx.parachainStaking
          .delegate(randomCandidate.address, DELEGATE_AMOUNT, 1, maxDelegationsPerDelegator + 1n)
          .signAsync(randomAccount, { nonce: nonce++ })
      )
    );
  });

  it("should support 100 delegations", async function () {
    // Additional check we have still have 1 delegation
    const delegatorState = await context.polkadotApi.query.parachainStaking.delegatorState(
      randomAccount.address
    );
    expect(delegatorState.unwrap().delegations.length).to.be.equal(
      Number(maxDelegationsPerDelegator),
      "Missing delegation"
    );
    // Only 1 over the 2 delegations has been revoked
    const locks = await context.polkadotApi.query.balances.locks(randomAccount.address);
    expect(locks.length).to.be.equal(1, "Missing lock");
    expect(locks[0].amount.toBigInt()).to.be.equal(DELEGATE_AMOUNT * maxDelegationsPerDelegator);
    expect(locks[0].id.toHuman().toString()).to.be.equal("stkngdel");
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

  it("should not be created for additional delegations", async function () {
    const locks = await context.polkadotApi.query.balances.locks(randomAccount.address);
    expect(locks.length).to.be.equal(
      1,
      `Unexpected number of locks: ${locks.map((l) => l.id.toHuman().toString()).join(` - `)}`
    );
  });

  it("should increase for additional delegations", async function () {
    const locks = await context.polkadotApi.query.balances.locks(randomAccount.address);
    expect(locks[0].id.toHuman().toString()).to.be.equal("stkngdel");
    expect(locks[0].amount.toBigInt(), `Unexpected amount for lock`).to.be.equal(
      2n * DELEGATE_AMOUNT
    );
  });
});

describeDevMoonbeam("Staking - Locks", (context) => {
  const randomAccount = generateKeyingPair();
  let additionalDelegators: KeyringPair[];

  before("Setup candidate & delegations", async function () {
    // Create the delegators to fill the lists
    additionalDelegators = new Array(
      context.polkadotApi.consts.parachainStaking.maxTopDelegationsPerCandidate.toNumber() +
        context.polkadotApi.consts.parachainStaking.maxBottomDelegationsPerCandidate.toNumber()
    )
      .fill(0)
      .map(() => generateKeyingPair());

    await context.createBlock(
      [randomAccount, ...additionalDelegators].map((account, i) =>
        context.polkadotApi.tx.balances
          .transfer(account.address, MIN_GLMR_STAKING * 2n + 1n * GLMR)
          .signAsync(alith, { nonce: i })
      )
    );
  });

  it("should get removed when bumped out of bottom list", async function () {
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegate(alith.address, MIN_GLMR_STAKING, 1, 1)
        .signAsync(randomAccount)
    );

    // Additional check
    const locks = await context.polkadotApi.query.balances.locks(randomAccount.address);
    expect(locks.length).to.be.equal(
      1,
      `Unexpected number of locks: ${locks.map((l) => l.id.toHuman().toString()).join(` - `)}`
    );

    await context.createBlock(
      [randomAccount, ...additionalDelegators].map((account, i) =>
        context.polkadotApi.tx.parachainStaking
          .delegate(alith.address, MIN_GLMR_STAKING + 1n * GLMR, i + 1, 1)
          .signAsync(account)
      )
    );

    const newLocks = await context.polkadotApi.query.balances.locks(randomAccount.address);
    expect(newLocks.length).to.be.equal(0, "Lock should have been removed after executing revoke");
  });
});

describeDevMoonbeam("Staking - Locks", (context) => {
  let bottomDelegators: KeyringPair[];
  let topDelegators: KeyringPair[];

  before("Setup candidate & delegations", async function () {
    // Create the delegators to fill the lists
    bottomDelegators = new Array(
      context.polkadotApi.consts.parachainStaking.maxBottomDelegationsPerCandidate.toNumber()
    )
      .fill(0)
      .map(() => generateKeyingPair());
    topDelegators = new Array(
      context.polkadotApi.consts.parachainStaking.maxTopDelegationsPerCandidate.toNumber()
    )
      .fill(0)
      .map(() => generateKeyingPair());

    await context.createBlock(
      [...bottomDelegators, ...topDelegators].map((account, i) =>
        context.polkadotApi.tx.balances
          .transfer(account.address, MIN_GLMR_STAKING * 2n + 1n * GLMR)
          .signAsync(alith, { nonce: i })
      )
    );
  });

  it("should be set for bottom and top list delegators", async function () {
    await context.createBlock(
      [...topDelegators].map((account, i) =>
        context.polkadotApi.tx.parachainStaking
          .delegate(alith.address, MIN_GLMR_STAKING + 1n * GLMR, i + 1, 1)
          .signAsync(account)
      )
    );
    await context.createBlock(
      [...bottomDelegators].map((account, i) =>
        context.polkadotApi.tx.parachainStaking
          .delegate(alith.address, MIN_GLMR_STAKING, topDelegators.length + i + 1, 1)
          .signAsync(account)
      )
    );

    const topLocks = await context.polkadotApi.query.balances.locks.multi(
      topDelegators.map((delegator) => delegator.address)
    );
    expect(
      topLocks.filter((lockSet) =>
        lockSet.find((lock) => lock.id.toHuman().toString() == "stkngdel")
      ).length
    ).to.equal(
      context.polkadotApi.consts.parachainStaking.maxTopDelegationsPerCandidate.toNumber()
    );

    const bottomLocks = await context.polkadotApi.query.balances.locks.multi(
      bottomDelegators.map((delegator) => delegator.address)
    );
    expect(
      bottomLocks.filter((lockSet) =>
        lockSet.find((lock) => lock.id.toHuman().toString() == "stkngdel")
      ).length
    ).to.equal(
      context.polkadotApi.consts.parachainStaking.maxBottomDelegationsPerCandidate.toNumber()
    );
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
    expect(locks[0].id.toHuman().toString()).to.be.equal("stkngcol");
  });
});

const testFeesForHotfixExtrinsicWithNumDelegators = async (
  context: DevTestContext,
  numDelegators: number
) => {
  let initialBalance = (
    await context.polkadotApi.query.system.account(baltathar.address)
  ).data.free.toBigInt();

  let accountIds = Array<string>(numDelegators).fill(alith.address);

  await context.createBlock(
    (context.polkadotApi.tx.parachainStaking as any)
      .hotfixMigrateDelegatorsFromReserveToLocks(accountIds)
      .signAsync(baltathar)
  );

  let afterBalance = (
    await context.polkadotApi.query.system.account(baltathar.address)
  ).data.free.toBigInt();

  const fee = initialBalance - afterBalance;
  return fee;
};

const testFeesForHotfixExtrinsicWithNumCollators = async (
  context: DevTestContext,
  numCollators: number
) => {
  let initialBalance = (
    await context.polkadotApi.query.system.account(baltathar.address)
  ).data.free.toBigInt();

  let accountIds = Array<string>(numCollators).fill(alith.address);

  await context.createBlock(
    (context.polkadotApi.tx.parachainStaking as any)
      .hotfixMigrateCollatorsFromReserveToLocks(accountIds)
      .signAsync(baltathar)
  );

  let afterBalance = (
    await context.polkadotApi.query.system.account(baltathar.address)
  ).data.free.toBigInt();

  const fee = initialBalance - afterBalance;
  return fee;
};

describeDevMoonbeam("Staking - Locks Hotfix Migration Extrinsics", (context) => {
  it("should have high fees", async function () {
    expect((await testFeesForHotfixExtrinsicWithNumDelegators(context, 1)) > 25_000_000_000_000n).to
      .be.true;
    expect(
      (await testFeesForHotfixExtrinsicWithNumDelegators(context, 100)) > 2_000_000_000_000_000n
    ).to.true;

    expect((await testFeesForHotfixExtrinsicWithNumCollators(context, 1)) > 25_000_000_000_000n).to
      .be.true;
    expect(
      (await testFeesForHotfixExtrinsicWithNumCollators(context, 100)) > 2_000_000_000_000_000n
    ).to.be.true;
  });
});

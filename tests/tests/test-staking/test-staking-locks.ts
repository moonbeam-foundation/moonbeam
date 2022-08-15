import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { alith, baltathar, generateKeyringPair } from "../../util/accounts";
import {
  execCouncilProposal,
  execTechnicalCommitteeProposal,
  notePreimage,
} from "../../util/governance";
import { GLMR, MIN_GLMR_STAKING, MIN_GLMR_DELEGATOR } from "../../util/constants";
import { describeDevMoonbeam, DevTestContext } from "../../util/setup-dev-tests";
import { KeyringPair } from "@polkadot/keyring/types";
import { expectOk } from "../../util/expect";
import { jumpRounds } from "../../util/block";
import { ExtrinsicCreation } from "../../util/substrate-rpc";

describeDevMoonbeam("Staking - Locks - join delegators", (context) => {
  const randomAccount = generateKeyringPair();

  before("setup account balance", async function () {
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.balances.transfer(randomAccount.address, MIN_GLMR_DELEGATOR + GLMR)
      )
    );
  });

  it('should set "stkngdel" when delegating', async function () {
    const { result } = await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegate(alith.address, MIN_GLMR_DELEGATOR, 10, 10)
        .signAsync(randomAccount)
    );
    const locks = await context.polkadotApi.query.balances.locks(randomAccount.address);
    expect(result.successful).to.be.true;
    expect(locks.length).to.be.equal(1, "Missing lock");
    expect(locks[0].amount.toBigInt()).to.be.equal(MIN_GLMR_DELEGATOR);
    expect(locks[0].id.toHuman().toString()).to.be.equal("stkngdel");
  });
});

describeDevMoonbeam("Staking - Locks - join candidates", (context) => {
  it('should set "stkngcol" when when joining candidates', async function () {
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .joinCandidates(MIN_GLMR_STAKING, 1)
        .signAsync(baltathar)
    );

    const locks = await context.polkadotApi.query.balances.locks(baltathar.address);
    expect(locks.length).to.be.equal(
      1,
      `Unexpected number of locks: ${locks.map((l) => l.id.toHuman()).join(` - `)}`
    );
    expect(locks[0].amount.toBigInt()).to.be.equal(MIN_GLMR_STAKING);
    expect(locks[0].id.toHuman()).to.be.equal("stkngcol");
  });
});

describeDevMoonbeam("Staking - Locks - delegator balance is locked", (context) => {
  const randomAccount = generateKeyringPair();

  before("setup account balance & delegation", async function () {
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.balances.transfer(randomAccount.address, MIN_GLMR_DELEGATOR + GLMR)
      )
    );
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .delegate(alith.address, MIN_GLMR_DELEGATOR, 0, 0)
          .signAsync(randomAccount)
      )
    );
  });

  it("should not be reusable for delegation", async function () {
    const { result } = await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegate(baltathar.address, MIN_GLMR_DELEGATOR, 10, 10)
        .signAsync(randomAccount)
    );
    expect(result.error.name.toString()).to.be.equal("InsufficientBalance");
  });
});

describeDevMoonbeam("Staking - Locks - candidate balance is locked", (context) => {
  const randomAccount = generateKeyringPair();

  before("setup account balance & delegation", async function () {
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.balances.transfer(randomAccount.address, MIN_GLMR_STAKING + GLMR)
      )
    );
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .joinCandidates(MIN_GLMR_STAKING, 1)
          .signAsync(randomAccount)
      )
    );
  });

  it("should not be reusable for transfer", async function () {
    const { result } = await context.createBlock(
      context.polkadotApi.tx.balances
        .transfer(alith.address, MIN_GLMR_STAKING)
        .signAsync(randomAccount)
    );
    expect(result.error.name.toString()).to.be.equal("LiquidityRestrictions");
  });
});

describeDevMoonbeam("Staking - Locks - democracy vote", (context) => {
  const randomAccount = generateKeyringPair();

  before("setup account balance & staking", async function () {
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.balances.transfer(randomAccount.address, MIN_GLMR_DELEGATOR + GLMR)
      )
    );
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .delegate(alith.address, MIN_GLMR_DELEGATOR, 0, 0)
          .signAsync(randomAccount)
      )
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
          Standard: { balance: MIN_GLMR_DELEGATOR, vote: { aye: true, conviction: 1 } },
        })
        .signAsync(randomAccount)
    );
    expect(result.successful).to.be.true;
    expect(result.events.find(({ event: { method } }) => method === "Voted")).to.not.be.undefined;
  });
});

describeDevMoonbeam("Staking - Locks - schedule revoke", (context) => {
  const randomAccount = generateKeyringPair();

  before("setup account balance", async function () {
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.balances.transfer(randomAccount.address, MIN_GLMR_DELEGATOR + GLMR)
      )
    );
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .delegate(alith.address, MIN_GLMR_DELEGATOR, 1, 0)
          .signAsync(randomAccount)
      )
    );
  });

  it("should stay locked after requesting a delegation revoke", async function () {
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .scheduleRevokeDelegation(alith.address)
          .signAsync(randomAccount)
      )
    );

    // Additional check
    const locks = await context.polkadotApi.query.balances.locks(randomAccount.address);
    expect(locks[0].id.toHuman().toString()).to.be.equal("stkngdel");
  });
});

describeDevMoonbeam("Staking - Locks - execute revoke", (context) => {
  const randomAccount = generateKeyringPair();

  before("setup account balance", async function () {
    await expectOk(
      context.createBlock([
        context.polkadotApi.tx.sudo.sudo(
          context.polkadotApi.tx.parachainStaking.setBlocksPerRound(5)
        ),
        context.polkadotApi.tx.balances.transfer(
          randomAccount.address,
          MIN_GLMR_DELEGATOR + 1n * GLMR
        ),
      ])
    );
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .delegate(alith.address, MIN_GLMR_DELEGATOR, 10, 10)
          .signAsync(randomAccount)
      )
    );
  });

  it("should be unlocked only after executing revoke delegation", async function () {
    this.timeout(20000);

    const lock = await context.polkadotApi.query.balances.locks(randomAccount.address);
    expect(lock.length).to.be.equal(1, "Lock should have been added");

    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .scheduleRevokeDelegation(alith.address)
          .signAsync(randomAccount)
      )
    );

    await jumpRounds(
      context,
      context.polkadotApi.consts.parachainStaking.revokeDelegationDelay.toNumber()
    );

    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .executeDelegationRequest(randomAccount.address, alith.address)
          .signAsync(randomAccount)
      )
    );

    const newLocks = await context.polkadotApi.query.balances.locks(randomAccount.address);
    expect(newLocks.length).to.be.equal(0, "Lock should have been removed after executing revoke");
  });
});

describeDevMoonbeam("Staking - Locks - multiple delegations single revoke", (context) => {
  const randomAccount = generateKeyringPair();

  before("setup candidate & delegations", async function () {
    this.timeout(20000);

    await expectOk(
      context.createBlock([
        context.polkadotApi.tx.sudo.sudo(
          context.polkadotApi.tx.parachainStaking.setBlocksPerRound(5)
        ),
        context.polkadotApi.tx.balances.transfer(randomAccount.address, 2n * MIN_GLMR_STAKING),
        context.polkadotApi.tx.parachainStaking
          .joinCandidates(MIN_GLMR_STAKING, 1)
          .signAsync(baltathar),
      ])
    );

    let nonce = await context.web3.eth.getTransactionCount(randomAccount.address);
    await expectOk(
      context.createBlock([
        context.polkadotApi.tx.parachainStaking
          .delegate(alith.address, MIN_GLMR_DELEGATOR, 10, 10)
          .signAsync(randomAccount, { nonce: nonce++ }),
        context.polkadotApi.tx.parachainStaking
          .delegate(baltathar.address, MIN_GLMR_DELEGATOR, 10, 10)
          .signAsync(randomAccount, { nonce: nonce++ }),
      ])
    );

    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .scheduleRevokeDelegation(alith.address)
          .signAsync(randomAccount)
      )
    );

    const locks = await context.polkadotApi.query.balances.locks(randomAccount.address);
    expect(locks.length).to.be.equal(1, "missing lock");
    expect(locks[0].amount.toBigInt()).to.equal(2n * MIN_GLMR_DELEGATOR);

    await jumpRounds(
      context,
      context.polkadotApi.consts.parachainStaking.revokeDelegationDelay.toNumber()
    );

    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .executeDelegationRequest(randomAccount.address, alith.address)
          .signAsync(randomAccount)
      )
    );
  });

  it("should be removed only after executing the last revoke delegation", async function () {
    // Additional check we still have 1 delegation
    const delegatorState = await context.polkadotApi.query.parachainStaking.delegatorState(
      randomAccount.address
    );
    expect(delegatorState.unwrap().delegations.length).to.be.equal(1, "Missing delegation");
    // Only 1 over the 2 delegations has been revoked
    const locks = await context.polkadotApi.query.balances.locks(randomAccount.address);
    expect(locks.length).to.be.equal(1, "Missing lock");
    expect(locks[0].amount.toBigInt()).to.be.equal(MIN_GLMR_DELEGATOR);
    expect(locks[0].id.toHuman().toString()).to.be.equal("stkngdel");
  });
});

describeDevMoonbeam("Staking - Locks - max delegations", (context) => {
  const randomAccount = generateKeyringPair();
  let randomCandidates: KeyringPair[];
  let maxDelegationsPerDelegator: bigint;

  before("setup candidate & delegations", async function () {
    maxDelegationsPerDelegator =
      context.polkadotApi.consts.parachainStaking.maxDelegationsPerDelegator.toBigInt();
    randomCandidates = new Array(Number(maxDelegationsPerDelegator))
      .fill(0)
      .map(() => generateKeyringPair());

    let alithNonce = await context.web3.eth.getTransactionCount(alith.address);
    await expectOk(
      context.createBlock([
        context.polkadotApi.tx.balances
          .transfer(randomAccount.address, (MIN_GLMR_DELEGATOR + GLMR) * maxDelegationsPerDelegator)
          .signAsync(alith, { nonce: alithNonce++ }),
        ...randomCandidates.map((randomCandidate) =>
          context.polkadotApi.tx.balances
            .transfer(randomCandidate.address, MIN_GLMR_STAKING + GLMR)
            .signAsync(alith, { nonce: alithNonce++ })
        ),
      ])
    );

    await expectOk(
      context.createBlock(
        randomCandidates.map((randomCandidate) =>
          context.polkadotApi.tx.parachainStaking
            .joinCandidates(MIN_GLMR_STAKING, maxDelegationsPerDelegator)
            .signAsync(randomCandidate)
        )
      )
    );

    const candidates = await context.polkadotApi.query.parachainStaking.candidateInfo.entries();
    expect(candidates.length).to.be.equal(
      Number(maxDelegationsPerDelegator) + 1,
      "Missing candidates"
    );

    let nonce = await context.web3.eth.getTransactionCount(randomAccount.address);
    await expectOk(
      context.createBlock(
        randomCandidates.map((randomCandidate) =>
          context.polkadotApi.tx.parachainStaking
            .delegate(
              randomCandidate.address,
              MIN_GLMR_DELEGATOR,
              1,
              maxDelegationsPerDelegator + 1n
            )
            .signAsync(randomAccount, { nonce: nonce++ })
        )
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
    // We should gave locked MIN_GLMR_DELEGATOR * maxDelegationsPerDelegator
    const locks = await context.polkadotApi.query.balances.locks(randomAccount.address);
    expect(locks.length).to.be.equal(1, "Missing lock");
    expect(locks[0].amount.toBigInt()).to.be.equal(MIN_GLMR_DELEGATOR * maxDelegationsPerDelegator);
    expect(locks[0].id.toHuman().toString()).to.be.equal("stkngdel");
  });
});

describeDevMoonbeam("Staking - Locks - multiple delegations single lock", (context) => {
  const randomAccount = generateKeyringPair();

  before("setup candidate & delegations", async function () {
    await expectOk(
      context.createBlock([
        context.polkadotApi.tx.balances.transfer(
          randomAccount.address,
          MIN_GLMR_STAKING * 2n + 1n * GLMR
        ),
        context.polkadotApi.tx.parachainStaking
          .joinCandidates(MIN_GLMR_STAKING, 1)
          .signAsync(baltathar),
      ])
    );

    let nonce = await context.web3.eth.getTransactionCount(randomAccount.address);
    await expectOk(
      context.createBlock([
        context.polkadotApi.tx.parachainStaking
          .delegate(alith.address, MIN_GLMR_DELEGATOR, 10, 10)
          .signAsync(randomAccount, { nonce: nonce++ }),
        context.polkadotApi.tx.parachainStaking
          .delegate(baltathar.address, MIN_GLMR_DELEGATOR, 10, 10)
          .signAsync(randomAccount, { nonce: nonce++ }),
      ])
    );
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
      2n * MIN_GLMR_DELEGATOR
    );
  });
});

describeDevMoonbeam("Staking - Locks - bottom delegator removed", (context) => {
  const randomAccount = generateKeyringPair();
  let additionalDelegators: KeyringPair[];

  before("setup candidate & delegations", async function () {
    this.timeout(20000);

    const maxDelegations =
      context.polkadotApi.consts.parachainStaking.maxTopDelegationsPerCandidate.toNumber() +
      context.polkadotApi.consts.parachainStaking.maxBottomDelegationsPerCandidate.toNumber();

    // Create the delegators to fill the lists
    additionalDelegators = new Array(maxDelegations).fill(0).map(() => generateKeyringPair());

    await expectOk(
      context.createBlock(
        [randomAccount, ...additionalDelegators].map((account, i) =>
          context.polkadotApi.tx.balances
            .transfer(account.address, MIN_GLMR_DELEGATOR + 10n * GLMR)
            .signAsync(alith, { nonce: i })
        )
      )
    );
  });

  it("should get removed when bumped out of bottom list", async function () {
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .delegate(alith.address, MIN_GLMR_DELEGATOR, 1, 1)
          .signAsync(randomAccount)
      )
    );

    // Additional check
    const locks = await context.polkadotApi.query.balances.locks(randomAccount.address);
    expect(locks.length).to.be.equal(
      1,
      `Unexpected number of locks: ${locks.map((l) => l.id.toHuman().toString()).join(` - `)}`
    );

    const txns = await [...additionalDelegators].map((account, i) =>
      context.polkadotApi.tx.parachainStaking
        .delegate(alith.address, MIN_GLMR_DELEGATOR + GLMR, i + 1, 1)
        .signAsync(account)
    );

    // this can no longer fit in one block
    const batchSize = 100;
    for (let i = 0; i < txns.length; i += batchSize) {
      await expectOk(context.createBlock(txns.slice(i, i + batchSize)));
    }

    const alithCandidateInfo = (
      (await context.polkadotApi.query.parachainStaking.candidateInfo(alith.address)) as any
    ).unwrap();
    expect(alithCandidateInfo.delegationCount.toNumber()).to.equal(additionalDelegators.length);

    const newLocks = await context.polkadotApi.query.balances.locks(randomAccount.address);
    expect(newLocks.length).to.be.equal(
      0,
      `Unexpected number of locks: ${newLocks
        .map((l) => `${l.id.toHuman().toString()}: ${l.amount.toHuman().toString()}`)
        .join(` - `)}`
    );
  });
});

describeDevMoonbeam("Staking - Locks - bottom and top delegations", (context) => {
  let bottomDelegators: KeyringPair[];
  let topDelegators: KeyringPair[];

  before("setup candidate & delegations", async function () {
    this.timeout(20000);

    // Create the delegators to fill the lists
    bottomDelegators = new Array(
      context.polkadotApi.consts.parachainStaking.maxBottomDelegationsPerCandidate.toNumber()
    )
      .fill(0)
      .map(() => generateKeyringPair());
    topDelegators = new Array(
      context.polkadotApi.consts.parachainStaking.maxTopDelegationsPerCandidate.toNumber()
    )
      .fill(0)
      .map(() => generateKeyringPair());

    await expectOk(
      context.createBlock(
        [...bottomDelegators, ...topDelegators].map((account, i) =>
          context.polkadotApi.tx.balances
            .transfer(account.address, MIN_GLMR_DELEGATOR + 2n * GLMR)
            .signAsync(alith, { nonce: i })
        )
      )
    );
  });

  it("should be set for bottom and top list delegators", async function () {
    await expectOk(
      context.createBlock(
        [...topDelegators].map((account, i) =>
          context.polkadotApi.tx.parachainStaking
            .delegate(alith.address, MIN_GLMR_DELEGATOR + 1n * GLMR, i + 1, 1)
            .signAsync(account)
        )
      )
    );
    await expectOk(
      context.createBlock(
        [...bottomDelegators].map((account, i) =>
          context.polkadotApi.tx.parachainStaking
            .delegate(alith.address, MIN_GLMR_DELEGATOR, topDelegators.length + i + 1, 1)
            .signAsync(account)
        )
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

import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { alith, baltathar, generateKeyringPair } from "../../../util/accounts";
import {
  execCouncilProposal,
  execTechnicalCommitteeProposal,
  notePreimage,
} from "../../../util/governance";
import {
  MICROGLMR,
  MILLIGLMR,
  GLMR,
  MIN_GLMR_STAKING,
  MIN_GLMR_DELEGATOR,
} from "../../../util/constants";
import { describeDevMoonbeam, DevTestContext } from "../../../util/setup-dev-tests";
import { KeyringPair } from "@polkadot/keyring/types";
import { expectOk } from "../../../util/expect";
import { jumpRounds } from "../../../util/block";
import { ExtrinsicCreation } from "../../../util/substrate-rpc";
import { chunk } from "../../../util/common";

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
    expect(result.error.name.toString()).to.be.equal('{"token":"Frozen"}');
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

    const proposal = context.polkadotApi.tx.parachainStaking.setParachainBondAccount(alith.address);
    const proposalHash = await notePreimage(context, proposal, alith);
    await execCouncilProposal(
      context,
      context.polkadotApi.tx.democracy.externalProposeMajority({
        LookUp: {
          hash: proposalHash,
          len: proposal.encodedLength,
        },
      } as any)
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
    this.timeout(40000);

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
    this.timeout(40000);

    await expectOk(
      context.createBlock([
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

    // We split the candidates since they won't fit in a single block
    for (const randomCandidatesChunk of chunk(randomCandidates, 20)) {
      await expectOk(
        context.createBlock(
          randomCandidatesChunk.map((randomCandidate) =>
            context.polkadotApi.tx.parachainStaking
              .joinCandidates(MIN_GLMR_STAKING, maxDelegationsPerDelegator)
              .signAsync(randomCandidate)
          )
        )
      );
    }

    const candidates = await context.polkadotApi.query.parachainStaking.candidateInfo.entries();
    expect(candidates.length).to.be.equal(
      Number(maxDelegationsPerDelegator) + 1,
      "Missing candidates"
    );

    let nonce = await context.web3.eth.getTransactionCount(randomAccount.address);
    for (const randomCandidatesChunk of chunk(randomCandidates, 20)) {
      await expectOk(
        context.createBlock(
          randomCandidatesChunk.map((randomCandidate) =>
            context.polkadotApi.tx.parachainStaking
              .delegateWithAutoCompound(
                randomCandidate.address,
                MIN_GLMR_DELEGATOR,
                100,
                1,
                1,
                maxDelegationsPerDelegator + 1n
              )
              .signAsync(randomAccount, { nonce: nonce++ })
          )
        )
      );
    }
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
          .delegateWithAutoCompound(alith.address, MIN_GLMR_DELEGATOR, 100, 10, 10, 10)
          .signAsync(randomAccount, { nonce: nonce++ }),
        context.polkadotApi.tx.parachainStaking
          .delegateWithAutoCompound(baltathar.address, MIN_GLMR_DELEGATOR, 100, 10, 10, 10)
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
          .delegateWithAutoCompound(alith.address, MIN_GLMR_DELEGATOR, 100, 1, 1, 1)
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
        .delegateWithAutoCompound(
          alith.address,
          MIN_GLMR_DELEGATOR + GLMR,
          100,
          additionalDelegators.length + 1,
          additionalDelegators.length + 1,
          1
        )
        .signAsync(account)
    );

    // this can no longer fit in one block
    for (const txnChunk of chunk(txns, 15)) {
      await expectOk(context.createBlock(txnChunk));
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
    const numBottomDelegations =
      context.polkadotApi.consts.parachainStaking.maxBottomDelegationsPerCandidate.toNumber();

    const numTopDelegations =
      context.polkadotApi.consts.parachainStaking.maxTopDelegationsPerCandidate.toNumber();

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
            .transfer(account.address, MIN_GLMR_DELEGATOR + 20n * GLMR)
            .signAsync(alith, { nonce: i })
        )
      )
    );
  });

  it("should be set for bottom and top list delegators", async function () {
    let tipOrdering = topDelegators.length + 1;
    let numDelegations = 0;
    for (const topDelegatorsChunk of chunk(topDelegators, 20)) {
      await expectOk(
        context.createBlock(
          [...topDelegatorsChunk].map((account, i) => {
            // add a tip such that the delegation ordering will be preserved,
            // e.g. the first txns sent will have the highest tip
            let tip = BigInt(tipOrdering--) * MILLIGLMR;
            return context.polkadotApi.tx.parachainStaking
              .delegateWithAutoCompound(
                alith.address,
                MIN_GLMR_DELEGATOR + 1n * GLMR,
                100,
                numDelegations,
                numDelegations++,
                1
              )
              .signAsync(account, { tip });
          })
        )
      );
    }

    // allow more block(s) for txns to be processed...
    // note: this only seems necessary when a tip is added, otherwise all 300 txns make it into a
    // single block. A tip is necessary if the txns are not otherwise executed in order of
    // submission, which is highly dependent on txpool prioritization logic.
    // TODO: it would be good to diagnose this further: why does adding a tip appear to reduce the
    // number of txns included?
    const numBlocksToWait = 1;
    let numBlocksWaited = 0;
    while (numBlocksWaited < numBlocksToWait) {
      await context.createBlock();
      const topLocks = await context.polkadotApi.query.balances.locks.multi(
        topDelegators.map((delegator) => delegator.address)
      );
      let numDelegatorLocks = topLocks.filter((lockSet) =>
        lockSet.find((lock) => lock.id.toHuman().toString() == "stkngdel")
      ).length;

      if (numDelegatorLocks < topDelegators.length) {
        numBlocksWaited += 1;
        expect(numBlocksWaited).to.be.lt(
          numBlocksToWait,
          "Top delegation extrinsics not included in time"
        );
      } else {
        expect(numDelegatorLocks).to.eq(topDelegators.length, "More delegations than expected");
        break;
      }
    }

    tipOrdering = bottomDelegators.length + 1;
    numDelegations = topDelegators.length;
    for (const bottomDelegatorsChunk of chunk(bottomDelegators, 20)) {
      await expectOk(
        context.createBlock(
          [...bottomDelegatorsChunk].map((account) => {
            // add a tip such that the delegation ordering will be preserved,
            // e.g. the first txns sent will have the highest tip
            let tip = BigInt(tipOrdering--) * MILLIGLMR;
            return context.polkadotApi.tx.parachainStaking
              .delegateWithAutoCompound(
                alith.address,
                MIN_GLMR_DELEGATOR,
                100,
                numDelegations,
                numDelegations++,
                1
              )
              .signAsync(account, { tip });
          })
        )
      );
    }

    // note that we don't need to wait for further blocks here because bottom delegations is much
    // smaller than top delegations, so all txns reliably fit within one block.
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

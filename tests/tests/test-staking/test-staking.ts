import "@moonbeam-network/api-augment";

import { u128 } from "@polkadot/types";
import {
  FrameSupportWeightsDispatchInfo,
  FrameSystemEventRecord,
  SpRuntimeDispatchError,
} from "@polkadot/types/lookup";
import { IEvent } from "@polkadot/types/types";
import { expect } from "chai";

import { alith, baltathar, ethan } from "../../util/accounts";
import {
  DEFAULT_GENESIS_MAPPING,
  DEFAULT_GENESIS_STAKING,
  GLMR,
  MIN_GLMR_DELEGATOR,
  MIN_GLMR_STAKING,
} from "../../util/constants";
import { describeDevMoonbeam, DevTestContext } from "../../util/setup-dev-tests";

describeDevMoonbeam("Staking - Genesis", (context) => {
  it("should match collator reserved bond reserved", async function () {
    const account = await context.polkadotApi.query.system.account(alith.address);
    const expectedReserved = DEFAULT_GENESIS_STAKING + DEFAULT_GENESIS_MAPPING;
    expect(account.data.reserved.toString()).to.equal(expectedReserved.toString());
  });

  it("should include collator from the specs", async function () {
    const collators = await context.polkadotApi.query.parachainStaking.selectedCandidates();
    expect(collators[0].toHex().toLowerCase()).equal(alith.address.toLowerCase());
  });

  it("should have collator state as defined in the specs", async function () {
    const collator = await context.polkadotApi.query.parachainStaking.candidateInfo(alith.address);
    expect(collator.unwrap().status.toString()).equal("Active");
  });

  it("should have inflation matching specs", async function () {
    const inflationInfo = await context.polkadotApi.query.parachainStaking.inflationConfig();
    // {
    //   expect: {
    //     min: '100.0000 kUNIT',
    //     ideal: '200.0000 kUNIT',
    //     max: '500.0000 kUNIT'
    //   },
    //  annual: {
    //     min: '4.00%',
    //     ideal: '5.00%',
    //     max: '5.00%',
    // },
    //   round: { min: '0.00%', ideal: '0.00%', max: '0.00%' }
    // }
    expect(inflationInfo["expect"]["min"].toBigInt()).to.eq(100_000n * GLMR);
    expect(inflationInfo["expect"]["ideal"].toBigInt()).to.eq(200_000n * GLMR);
    expect(inflationInfo["expect"]["max"].toBigInt()).to.eq(500_000n * GLMR);
    expect(inflationInfo.annual.min.toHuman()).to.eq("4.00%");
    expect(inflationInfo.annual.ideal.toHuman()).to.eq("5.00%");
    expect(inflationInfo.annual.max.toHuman()).to.eq("5.00%");
    expect(inflationInfo.round.min.toHuman()).to.eq("0.00%");
    expect(inflationInfo.round.min.toNumber()).to.eq(8949); // 4% / blocks per year * 10^9
    expect(inflationInfo.round.ideal.toHuman()).to.eq("0.00%");
    expect(inflationInfo.round.ideal.toNumber()).to.eq(11132); // 5% / blocks per year * 10^9
    expect(inflationInfo.round.max.toHuman()).to.eq("0.00%");
    expect(inflationInfo.round.max.toNumber()).to.eq(11132); // 5% / blocks per year * 10^9
  });
});

describeDevMoonbeam("Staking - Join Candidates", (context) => {
  before("should join as a candidate", async () => {
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, 1).signAsync(ethan)
    );
  });

  afterEach("cleanup candidate leave request", async () => {
    let candidateCount = (await context.polkadotApi.query.parachainStaking.candidatePool()).length;
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking.cancelLeaveCandidates(candidateCount).signAsync(ethan)
    );
  });

  it("should successfully call joinCandidates", async function () {
    let candidatesAfter = await context.polkadotApi.query.parachainStaking.candidatePool();
    expect(candidatesAfter.length).to.equal(2, "new candidate should have been added");
    expect(candidatesAfter[1].owner.toString()).to.equal(
      ethan.address,
      "new candidate ethan should have been added"
    );
    expect(candidatesAfter[1].amount.toBigInt()).to.equal(
      1000n * GLMR,
      "new candidate ethan should have been added (wrong amount)"
    );
  });

  it("should successfully schedule leave candidates", async function () {
    let candidatesAfter = await context.polkadotApi.query.parachainStaking.candidatePool();
    expect(candidatesAfter.length).to.equal(2, "new candidate should have been added");

    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .scheduleLeaveCandidates(candidatesAfter.length)
        .signAsync(ethan)
    );

    const candidateState = (
      await context.polkadotApi.query.parachainStaking.candidateInfo(ethan.address)
    ).unwrap();
    const currentRound = (
      await context.polkadotApi.query.parachainStaking.round()
    ).current.toNumber();
    const roundDelay = context.polkadotApi.consts.parachainStaking.leaveCandidatesDelay.toNumber();

    expect(candidateState.status.isLeaving).to.be.true;
    expect(candidateState.status.asLeaving.toNumber()).to.equal(currentRound + roundDelay);
  });

  it("should successfully execute schedule leave candidates at correct round", async function () {
    this.timeout(20000);

    let candidatesAfter = await context.polkadotApi.query.parachainStaking.candidatePool();
    expect(candidatesAfter.length).to.equal(2, "new candidate should have been added");

    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .scheduleLeaveCandidates(candidatesAfter.length)
        .signAsync(ethan)
    );

    const candidateState = (
      await context.polkadotApi.query.parachainStaking.candidateInfo(ethan.address)
    ).unwrap();
    expect(candidateState.status.isLeaving).to.be.true;

    const whenRound = candidateState.status.asLeaving.toNumber();
    await jumpToRound(context, whenRound - 1);

    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .executeLeaveCandidates(ethan.address, candidateState.delegationCount)
        .signAsync(ethan)
    );
    const extrinsicResult = await getExtrinsicResult(
      context,
      "parachainStaking",
      "executeLeaveCandidates"
    );
    expect(extrinsicResult).to.equal("CandidateCannotLeaveYet");

    await jumpToRound(context, whenRound);
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .executeLeaveCandidates(ethan.address, candidateState.delegationCount)
        .signAsync(ethan)
    );
    const candidateStateAfter = await context.polkadotApi.query.parachainStaking.candidateInfo(
      ethan.address
    );
    expect(candidateStateAfter.isNone).to.be.true;
  });
});

describeDevMoonbeam("Staking - Join Delegators", (context) => {
  before("should successfully call delegate on Alith", async function () {
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegate(alith.address, MIN_GLMR_DELEGATOR, 0, 0)
        .signAsync(ethan)
    );
  });

  afterEach("cleanup delegator leave request", async () => {
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking.cancelLeaveDelegators().signAsync(ethan)
    );
  });

  it("should have successfully delegated stake to Alith", async function () {
    const delegatorsAfter = (
      await context.polkadotApi.query.parachainStaking.delegatorState(ethan.address)
    ).unwrap();
    expect(delegatorsAfter.delegations[0].owner.toString()).to.equal(
      alith.address,
      "new delegation to alith should have been added"
    );
    expect(delegatorsAfter.delegations[0].amount.toBigInt()).to.equal(
      1n * GLMR,
      "delegation amount to alith should be 1"
    );
  });

  it("should successfully schedule leave delegators", async function () {
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking.scheduleLeaveDelegators().signAsync(ethan)
    );

    const delegatorState = (
      await context.polkadotApi.query.parachainStaking.delegatorState(ethan.address)
    ).unwrap();
    const currentRound = (
      await context.polkadotApi.query.parachainStaking.round()
    ).current.toNumber();
    const roundDelay = context.polkadotApi.consts.parachainStaking.leaveDelegatorsDelay.toNumber();

    for await (const delegation of delegatorState.delegations) {
      const scheduledRequests =
        (await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(
          delegation.owner
        )) as unknown as any[];
      const revokeRequest = scheduledRequests.find(
        (req) => req.delegator.eq(ethan.address) && req.action.isRevoke
      );
      expect(revokeRequest).to.not.be.undefined;
      expect(revokeRequest.whenExecutable.toNumber()).to.equal(currentRound + roundDelay);
    }
  });

  it("should successfully execute schedule leave delegators at correct round", async function () {
    this.timeout(20000);

    await context.createBlock(
      context.polkadotApi.tx.parachainStaking.scheduleLeaveDelegators().signAsync(ethan)
    );

    const delegatorState = (
      await context.polkadotApi.query.parachainStaking.delegatorState(ethan.address)
    ).unwrap();
    const scheduledRequests =
      (await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(
        alith.address
      )) as unknown as any[];
    const revokeRequest = scheduledRequests.find(
      (req) => req.delegator.eq(ethan.address) && req.action.isRevoke
    );
    expect(revokeRequest).to.not.be.undefined;

    const whenRound = revokeRequest.whenExecutable.toNumber();
    await jumpToRound(context, whenRound - 1);

    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .executeLeaveDelegators(ethan.address, delegatorState.delegations.length)
        .signAsync(ethan)
    );
    const extrinsicResult = await getExtrinsicResult(
      context,
      "parachainStaking",
      "executeLeaveDelegators"
    );
    expect(extrinsicResult).to.equal("DelegatorCannotLeaveYet");

    await jumpToRound(context, whenRound);
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .executeLeaveDelegators(ethan.address, delegatorState.delegations.length)
        .signAsync(ethan)
    );
    const delegatorStateAfter = await context.polkadotApi.query.parachainStaking.delegatorState(
      ethan.address
    );
    expect(delegatorStateAfter.isNone).to.be.true;
  });
});

describeDevMoonbeam("Staking - Delegation Requests", (context) => {
  const numberToHex = (n: BigInt): string => `0x${n.toString(16).padStart(32, "0")}`;
  const DELEGATION = MIN_GLMR_DELEGATOR * 5n;

  beforeEach("should successfully call delegate on Alith", async () => {
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegate(alith.address, DELEGATION, 0, 0)
        .signAsync(ethan)
    );
  });

  afterEach("should clean up delegation requests", async () => {
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .cancelDelegationRequest(alith.address)
        .signAsync(ethan)
    );
  });

  it("should successfully schedule revoke", async function () {
    const delegationRequestsBefore =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(alith.address);
    expect(delegationRequestsBefore.toJSON()).to.be.empty;

    const currentRound = (
      await context.polkadotApi.query.parachainStaking.round()
    ).current.toNumber();

    // schedule revoke
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .scheduleRevokeDelegation(alith.address)
        .signAsync(ethan)
    );

    const delegationRequestsAfter =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(alith.address);
    const roundDelay = context.polkadotApi.consts.parachainStaking.revokeDelegationDelay.toNumber();
    expect(delegationRequestsAfter.toJSON()).to.deep.equal([
      {
        delegator: ethan.address,
        whenExecutable: currentRound + roundDelay,
        action: {
          revoke: numberToHex(DELEGATION),
        },
      },
    ]);
  });

  it("should successfully cancel revoke", async function () {
    const delegationRequestsBefore =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(alith.address);
    expect(delegationRequestsBefore.toJSON()).to.be.empty;

    const currentRound = (
      await context.polkadotApi.query.parachainStaking.round()
    ).current.toNumber();

    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .scheduleRevokeDelegation(alith.address)
        .signAsync(ethan)
    );
    const delegationRequestsAfterSchedule =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(alith.address);
    const roundDelay = context.polkadotApi.consts.parachainStaking.revokeDelegationDelay.toNumber();
    expect(delegationRequestsAfterSchedule.toJSON()).to.deep.equal([
      {
        delegator: ethan.address,
        whenExecutable: currentRound + roundDelay,
        action: {
          revoke: numberToHex(DELEGATION),
        },
      },
    ]);

    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .cancelDelegationRequest(alith.address)
        .signAsync(ethan)
    );

    const delegationRequestsAfterCancel =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(alith.address);
    expect(delegationRequestsAfterCancel).to.be.empty;
  });

  it("should not execute revoke before target round", async function () {
    this.timeout(50000);

    const currentRound = (
      await context.polkadotApi.query.parachainStaking.round()
    ).current.toNumber();

    // schedule revoke
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .scheduleRevokeDelegation(alith.address)
        .signAsync(ethan)
    );
    const delegationRequests =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(alith.address);
    expect(delegationRequests.toJSON()).to.not.be.empty;

    // jump to a round before the actual executable Round
    await jumpToRound(context, delegationRequests[0].whenExecutable.toNumber() - 1);

    // execute revoke

    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .executeDelegationRequest(ethan.address, alith.address)
        .signAsync(ethan)
    );
    const extrinsicResult = await getExtrinsicResult(
      context,
      "parachainStaking",
      "executeDelegationRequest"
    );
    expect(extrinsicResult).to.equal("PendingDelegationRequestNotDueYet");

    const { delegations: delegationsAfter } = (
      await context.polkadotApi.query.parachainStaking.delegatorState(ethan.address)
    ).unwrap();
    const delegationRequestsAfter =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(alith.address);

    expect(delegationsAfter.toJSON()).to.deep.equal([
      {
        owner: alith.address,
        amount: numberToHex(DELEGATION),
      },
    ]);
    const roundDelay = context.polkadotApi.consts.parachainStaking.revokeDelegationDelay.toNumber();
    expect(delegationRequestsAfter.toJSON()).to.deep.equal([
      {
        delegator: ethan.address,
        whenExecutable: currentRound + roundDelay,
        action: {
          revoke: numberToHex(DELEGATION),
        },
      },
    ]);
  });

  it("should successfully execute revoke", async function () {
    this.timeout(20000);

    // schedule revoke
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .scheduleRevokeDelegation(alith.address)
        .signAsync(ethan)
    );
    const delegationRequests =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(alith.address);
    expect(delegationRequests.toJSON()).to.not.be.empty;

    // jump to executable Round
    await jumpToRound(context, delegationRequests[0].whenExecutable.toNumber());

    // execute revoke
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .executeDelegationRequest(ethan.address, alith.address)
        .signAsync(ethan)
    );

    const delegationsAfter = await context.polkadotApi.query.parachainStaking.delegatorState(
      ethan.address
    );
    const delegationRequestsAfter =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(alith.address);
    // last delegation revoked, so delegator marked as leaving
    expect(delegationsAfter.isNone).to.be.true;
    expect(delegationRequestsAfter.toJSON()).to.be.empty;
  });

  it("should successfully schedule bond less", async function () {
    const delegationRequestsBefore =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(alith.address);
    expect(delegationRequestsBefore.toJSON()).to.be.empty;

    const currentRound = (
      await context.polkadotApi.query.parachainStaking.round()
    ).current.toNumber();

    // schedule bond less
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .scheduleDelegatorBondLess(alith.address, 10n)
        .signAsync(ethan)
    );
    const delegationRequestsAfter =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(alith.address);
    const roundDelay =
      context.polkadotApi.consts.parachainStaking.delegationBondLessDelay.toNumber();
    expect(delegationRequestsAfter.toJSON()).to.deep.equal([
      {
        delegator: ethan.address,
        whenExecutable: currentRound + roundDelay,
        action: {
          decrease: 10,
        },
      },
    ]);
  });

  it("should successfully cancel bond less", async function () {
    const delegationRequestsBefore =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(alith.address);
    expect(delegationRequestsBefore.toJSON()).to.be.empty;

    const currentRound = (
      await context.polkadotApi.query.parachainStaking.round()
    ).current.toNumber();

    const LESS_AMOUNT = 10;
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .scheduleDelegatorBondLess(alith.address, LESS_AMOUNT)
        .signAsync(ethan)
    );
    const delegationRequestsAfterSchedule =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(alith.address);
    const roundDelay =
      context.polkadotApi.consts.parachainStaking.delegationBondLessDelay.toNumber();
    expect(delegationRequestsAfterSchedule.toJSON()).to.deep.equal([
      {
        delegator: ethan.address,
        whenExecutable: currentRound + roundDelay,
        action: {
          decrease: LESS_AMOUNT,
        },
      },
    ]);

    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .cancelDelegationRequest(alith.address)
        .signAsync(ethan)
    );

    const delegationRequestsAfterCancel =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(alith.address);
    expect(delegationRequestsAfterCancel.toJSON()).to.be.empty;
  });

  it("should not execute bond less before target round", async function () {
    this.timeout(50000);

    const currentRound = (
      await context.polkadotApi.query.parachainStaking.round()
    ).current.toNumber();

    // schedule bond less
    const LESS_AMOUNT = 10;
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .scheduleDelegatorBondLess(alith.address, LESS_AMOUNT)
        .signAsync(ethan)
    );
    const delegationRequests =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(alith.address);
    expect(delegationRequests.toJSON()).to.not.be.empty;

    // jump to a round before the actual executable Round
    await jumpToRound(context, delegationRequests[0].whenExecutable.toNumber() - 1);

    // execute bond less

    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .executeDelegationRequest(ethan.address, alith.address)
        .signAsync(ethan)
    );
    const extrinsicResult = await getExtrinsicResult(
      context,
      "parachainStaking",
      "executeDelegationRequest"
    );
    expect(extrinsicResult).to.equal("PendingDelegationRequestNotDueYet");

    const { delegations: delegationsAfter } = (
      await context.polkadotApi.query.parachainStaking.delegatorState(ethan.address)
    ).unwrap();
    const delegationRequestsAfter =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(alith.address);

    expect(delegationsAfter.toJSON()).to.deep.equal([
      {
        owner: alith.address,
        amount: numberToHex(DELEGATION),
      },
    ]);

    const roundDelay =
      context.polkadotApi.consts.parachainStaking.delegationBondLessDelay.toNumber();
    expect(delegationRequestsAfter.toJSON()).to.deep.equal([
      {
        delegator: ethan.address,
        whenExecutable: currentRound + roundDelay,
        action: {
          decrease: LESS_AMOUNT,
        },
      },
    ]);
  });

  it("should successfully execute bond less", async function () {
    this.timeout(20000);

    // schedule bond less
    const LESS_AMOUNT = 10;
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .scheduleDelegatorBondLess(alith.address, LESS_AMOUNT)
        .signAsync(ethan)
    );
    const delegationRequests =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(alith.address);
    expect(delegationRequests).to.not.be.empty;

    // jump to executable Round
    await jumpToRound(context, delegationRequests[0].whenExecutable.toNumber());

    // execute bond less
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .executeDelegationRequest(ethan.address, alith.address)
        .signAsync(ethan)
    );

    const {
      delegations: [firstDelegationAfter, ..._],
    } = (await context.polkadotApi.query.parachainStaking.delegatorState(ethan.address)).unwrap();
    const delegationRequestsAfter =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(alith.address);
    expect(firstDelegationAfter.toJSON()).to.deep.equal({
      owner: alith.address,
      amount: numberToHex(DELEGATION - BigInt(LESS_AMOUNT)),
    });
    expect(delegationRequestsAfter.toJSON()).to.be.empty;
  });

  it("should successfully remove scheduled requests on collator leave", async function () {
    this.timeout(20000);

    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .joinCandidates(MIN_GLMR_STAKING, 1)
        .signAsync(baltathar)
    );

    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegate(baltathar.address, DELEGATION, 0, 1)
        .signAsync(ethan)
    );

    const delegationRequestsBefore =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(
        baltathar.address
      );
    expect(delegationRequestsBefore.isEmpty).to.be.true;

    // schedule bond less

    await context.createBlock([
      context.polkadotApi.tx.parachainStaking
        .scheduleDelegatorBondLess(baltathar.address, 10n)
        .signAsync(ethan),
      context.polkadotApi.tx.parachainStaking.scheduleLeaveCandidates(2).signAsync(baltathar),
    ]);

    const collatorState = await context.polkadotApi.query.parachainStaking.candidateInfo(
      baltathar.address
    );
    await jumpToRound(context, collatorState.unwrap().status.asLeaving.toNumber());

    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .executeLeaveCandidates(baltathar.address, 1)
        .signAsync(ethan)
    );
    const delegationRequestsAfter =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(
        baltathar.address
      );
    expect(delegationRequestsAfter.toJSON()).to.be.empty;
    const delegationRequestsKeysAfter = (
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests.keys()
    ).map(({ args: [accountId] }) => accountId.toString());
    expect(delegationRequestsKeysAfter).to.not.contain(baltathar.address);
  });

  it("should successfully remove scheduled requests on delegator leave", async function () {
    this.timeout(20000);

    const delegationRequestsBefore =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(alith.address);
    expect(delegationRequestsBefore.toJSON()).to.be.empty;

    // schedule bond less
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .scheduleDelegatorBondLess(alith.address, 10n)
        .signAsync(ethan)
    );
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking.scheduleLeaveDelegators().signAsync(ethan)
    );

    const delegatorState = await context.polkadotApi.query.parachainStaking.delegatorState(
      ethan.address
    );
    const scheduledRequests =
      (await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(
        alith.address
      )) as unknown as any[];
    const revokeRequest = scheduledRequests.find(
      (req) => req.delegator.eq(ethan.address) && req.action.isRevoke
    );
    expect(revokeRequest).to.not.be.undefined;
    await jumpToRound(context, revokeRequest.whenExecutable.toNumber());

    const block = await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .executeLeaveDelegators(ethan.address, 1)
        .signAsync(ethan)
    );
    const delegationRequestsAfter =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(alith.address);
    expect(delegationRequestsAfter.toJSON()).to.be.empty;
    const leaveEvents = await getEventsAtFilter(context, block.block.hash.toString(), (event) => {
      if (context.polkadotApi.events.parachainStaking.DelegatorLeft.is(event.event)) {
        return {
          account: event.event.data[0].toString(),
        };
      }
    });
    expect(leaveEvents).to.deep.equal([
      {
        account: "0xFf64d3F6efE2317EE2807d223a0Bdc4c0c49dfDB",
      },
    ]);
  });
});

describeDevMoonbeam("Staking - Bond More", (context) => {
  const DELEGATION = MIN_GLMR_DELEGATOR * 5n;
  before("should successfully call delegate on Alith", async () => {
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegate(alith.address, DELEGATION, 0, 0)
        .signAsync(ethan)
    );
  });

  afterEach("should clean up delegation requests", async () => {
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .cancelDelegationRequest(alith.address)
        .signAsync(ethan)
    );
  });

  it("should allow bond more when no delgation request scheduled", async function () {
    const bondAmountBefore = (
      await context.polkadotApi.query.parachainStaking.delegatorState(ethan.address)
    ).unwrap().total;

    // schedule bond less
    const increaseAmount = 5;
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegatorBondMore(alith.address, increaseAmount)
        .signAsync(ethan)
    );

    const bondAmountAfter = (
      await context.polkadotApi.query.parachainStaking.delegatorState(ethan.address)
    ).unwrap().total;
    expect(bondAmountAfter.eq(bondAmountBefore.addn(increaseAmount))).to.be.true;
  });

  it("should allow bond more when bond less schedule", async function () {
    const bondAmountBefore = (
      await context.polkadotApi.query.parachainStaking.delegatorState(ethan.address)
    ).unwrap().total;

    // schedule bond less
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .scheduleDelegatorBondLess(alith.address, 10n)
        .signAsync(ethan)
    );

    const increaseAmount = 5;
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegatorBondMore(alith.address, increaseAmount)
        .signAsync(ethan)
    );

    const bondAmountAfter = (
      await context.polkadotApi.query.parachainStaking.delegatorState(ethan.address)
    ).unwrap().total;
    expect(bondAmountAfter.eq(bondAmountBefore.addn(increaseAmount))).to.be.true;
  });

  it("should not allow bond more when revoke schedule", async function () {
    const bondAmountBefore = (
      await context.polkadotApi.query.parachainStaking.delegatorState(ethan.address)
    ).unwrap().total;

    // schedule bond less
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .scheduleRevokeDelegation(alith.address)
        .signAsync(ethan)
    );

    const increaseAmount = 5n;
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegatorBondMore(alith.address, increaseAmount)
        .signAsync(ethan)
    );

    const extrinsicError = await getExtrinsicResult(
      context,
      "parachainStaking",
      "delegatorBondMore"
    );
    expect(extrinsicError).to.equal("PendingDelegationRevoke");
    const bondAmountAfter = (
      await context.polkadotApi.query.parachainStaking.delegatorState(ethan.address)
    ).unwrap().total;
    expect(bondAmountAfter.eq(bondAmountBefore)).to.be.true;
  });
});

describeDevMoonbeam("Staking - Rewards", (context) => {
  const EXTRA_BOND_AMOUNT = 1_000_000_000_000_000_000n;
  const BOND_AMOUNT = MIN_GLMR_STAKING + EXTRA_BOND_AMOUNT;

  beforeEach("should successfully call delegate on Alith", async () => {
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegate(alith.address, BOND_AMOUNT, 0, 0)
        .signAsync(ethan)
    );
    const currentRound = await context.polkadotApi.query.parachainStaking.round();
    await jumpToRound(context, currentRound.current.addn(1).toNumber());
  });

  afterEach("should clean up delegation requests", async () => {
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .cancelDelegationRequest(alith.address)
        .signAsync(ethan)
    );
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking.cancelLeaveDelegators().signAsync(ethan)
    );
  });

  it("should reward delegators without scheduled requests", async function () {
    this.timeout(20000);

    const rewardDelay = context.polkadotApi.consts.parachainStaking.rewardPaymentDelay;
    const currentRound = (await context.polkadotApi.query.parachainStaking.round()).current;
    const blockHash = await jumpToRound(context, currentRound.add(rewardDelay).toNumber());
    let rewardedEvents = await getRewardedEventsAt(context, blockHash);

    expect(
      rewardedEvents.some(({ account }) => account == ethan.address),
      "delegator was not rewarded"
    ).to.be.true;
  });

  it("should not reward delegator if leave scheduled", async function () {
    this.timeout(20000);

    await context.createBlock(
      context.polkadotApi.tx.parachainStaking.scheduleLeaveDelegators().signAsync(ethan)
    );

    const rewardDelay = context.polkadotApi.consts.parachainStaking.rewardPaymentDelay;
    const currentRound = (await context.polkadotApi.query.parachainStaking.round()).current;
    const blockHash = await jumpToRound(context, currentRound.add(rewardDelay).addn(1).toNumber());
    let rewardedEvents = await getRewardedEventsAt(context, blockHash);
    expect(
      rewardedEvents.some(({ account }) => account == ethan.address),
      "delegator was incorrectly rewarded"
    ).to.be.false;
  });

  it("should not reward delegator if revoke scheduled", async function () {
    this.timeout(20000);

    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .scheduleRevokeDelegation(alith.address)
        .signAsync(ethan)
    );

    const rewardDelay = context.polkadotApi.consts.parachainStaking.rewardPaymentDelay;
    const currentRound = (await context.polkadotApi.query.parachainStaking.round()).current;
    const blockHash = await jumpToRound(context, currentRound.add(rewardDelay).addn(1).toNumber());

    let rewardedEvents = await getRewardedEventsAt(context, blockHash);
    expect(
      rewardedEvents.some(({ account }) => account == ethan.address),
      "delegator was incorrectly rewarded"
    ).to.be.false;
  });

  it("should reward delegator after deducting pending bond decrease", async function () {
    this.timeout(20000);

    await context.createBlock([
      context.polkadotApi.tx.parachainStaking
        .delegate(alith.address, BOND_AMOUNT, 1, 0)
        .signAsync(baltathar),
      context.polkadotApi.tx.parachainStaking
        .scheduleDelegatorBondLess(alith.address, EXTRA_BOND_AMOUNT)
        .signAsync(ethan),
    ]);

    const rewardDelay = context.polkadotApi.consts.parachainStaking.rewardPaymentDelay;
    const currentRound = (await context.polkadotApi.query.parachainStaking.round()).current;
    const blockHash = await jumpToRound(context, currentRound.add(rewardDelay).addn(1).toNumber());
    let rewardedEvents = await getRewardedEventsAt(context, blockHash);
    let rewardedEthan = rewardedEvents.find(({ account }) => account == ethan.address);
    let rewardedBalathar = rewardedEvents.find(({ account }) => account == baltathar.address);
    expect(rewardedEthan).is.not.undefined;
    expect(rewardedBalathar).is.not.undefined;
    expect(
      rewardedBalathar.amount.gt(rewardedEthan.amount),
      `Ethan's reward ${rewardedEthan.amount} was not less than Balathar's \
      reward ${rewardedBalathar.amount}`
    ).is.true;
  });
});

async function jumpToRound(context: DevTestContext, round: Number): Promise<string | null> {
  let lastBlockHash = null;
  while (true) {
    const currentRound = (
      await context.polkadotApi.query.parachainStaking.round()
    ).current.toNumber();
    if (currentRound == round) {
      return lastBlockHash;
    }
    lastBlockHash = (await context.createBlock()).block.hash.toString();
  }
}

async function getExtrinsicResult(
  context: DevTestContext,
  pallet: string,
  call: string
): Promise<string | null> {
  const signedBlock = await context.polkadotApi.rpc.chain.getBlock();
  const apiAt = await context.polkadotApi.at(signedBlock.block.header.hash);
  const allEvents = await apiAt.query.system.events();

  const extrinsicIndex = signedBlock.block.extrinsics.findIndex(
    (ext) => pallet == ext.method.section && call === ext.method.method
  );
  if (extrinsicIndex === -1) {
    return null;
  }

  const failedEvent = allEvents.find(
    ({ phase, event }) =>
      phase.isApplyExtrinsic &&
      phase.asApplyExtrinsic.eq(extrinsicIndex) &&
      context.polkadotApi.events.system.ExtrinsicFailed.is(event)
  );
  if (!failedEvent) {
    return null;
  }

  const event: IEvent<[SpRuntimeDispatchError, FrameSupportWeightsDispatchInfo]> =
    failedEvent.event as any;
  const [dispatchError, _dispatchInfo] = event.data;
  if (dispatchError.isModule) {
    const decodedError = context.polkadotApi.registry.findMetaError(dispatchError.asModule);
    return decodedError.name;
  }

  return dispatchError.toString();
}

async function getEventsAtFilter(
  context: DevTestContext,
  blockHash: string,
  cb: (event: FrameSystemEventRecord) => object
): Promise<Array<object>> {
  const signedBlock = await context.polkadotApi.rpc.chain.getBlock(blockHash);
  const apiAt = await context.polkadotApi.at(signedBlock.block.header.hash);

  let events = [];
  for await (const event of await apiAt.query.system.events()) {
    const data = cb(event);
    if (data) {
      events.push(data);
    }
  }

  return events;
}

async function getRewardedEventsAt(
  context: DevTestContext,
  blockHash: string
): Promise<Array<{ account: string; amount: u128 }>> {
  const signedBlock = await context.polkadotApi.rpc.chain.getBlock(blockHash);
  const apiAt = await context.polkadotApi.at(signedBlock.block.header.hash);

  let rewardedEvents: Array<{ account: string; amount: u128 }> = [];
  for await (const event of await apiAt.query.system.events()) {
    if (context.polkadotApi.events.parachainStaking.Rewarded.is(event.event)) {
      rewardedEvents.push({
        account: event.event.data[0].toString(),
        amount: event.event.data[1] as any,
      });
    }
  }

  return rewardedEvents;
}

import "@polkadot/api-augment";
import "@moonbeam-network/api-augment";
import {
  SpRuntimeDispatchError,
  FrameSupportWeightsDispatchInfo,
  FrameSystemEventRecord,
} from "@polkadot/types/lookup";
import type { IsEvent } from "@polkadot/types/metadata/decorate/types";
import { expect } from "chai";
import Keyring from "@polkadot/keyring";
import {
  DEFAULT_GENESIS_MAPPING,
  DEFAULT_GENESIS_STAKING,
  COLLATOR_ACCOUNT,
  ETHAN_PRIVKEY,
  MIN_GLMR_STAKING,
  ETHAN,
  ALITH,
  MIN_GLMR_DELEGATOR,
  GLMR,
  BALTATHAR_PRIV_KEY,
  BALTATHAR,
  ALITH_ADDRESS,
} from "../util/constants";
import { describeDevMoonbeam, DevTestContext } from "../util/setup-dev-tests";
import { KeyringPair } from "@substrate/txwrapper-core";
import { AnyTuple, IEvent } from "@polkadot/types/types";
import { GenericEventData, u128, Vec } from "@polkadot/types";
import { BN } from "@polkadot/util";
import { StringMappingType } from "typescript";
import { AugmentedEvent } from "@polkadot/api/types";

describeDevMoonbeam("Staking - Genesis", (context) => {
  it("should match collator reserved bond reserved", async function () {
    const account = await context.polkadotApi.query.system.account(COLLATOR_ACCOUNT);
    const expectedReserved = DEFAULT_GENESIS_STAKING + DEFAULT_GENESIS_MAPPING;
    expect(account.data.reserved.toString()).to.equal(expectedReserved.toString());
  });

  it("should include collator from the specs", async function () {
    const collators = await context.polkadotApi.query.parachainStaking.selectedCandidates();
    expect(collators[0].toHex().toLowerCase()).equal(COLLATOR_ACCOUNT.toLowerCase());
  });

  it("should have collator state as defined in the specs", async function () {
    const collator = await context.polkadotApi.query.parachainStaking.candidateInfo(
      COLLATOR_ACCOUNT
    );
    expect(collator.toHuman()["status"]).equal("Active");
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
    expect(inflationInfo.toHuman()["annual"]["min"]).to.eq("4.00%");
    expect(inflationInfo.toHuman()["annual"]["ideal"]).to.eq("5.00%");
    expect(inflationInfo.toHuman()["annual"]["max"]).to.eq("5.00%");
    expect(inflationInfo.toHuman()["round"]["min"]).to.eq("0.00%");
    expect(Number(inflationInfo["round"]["min"])).to.eq(8949); // 4% / blocks per year * 10^9
    expect(inflationInfo.toHuman()["round"]["ideal"]).to.eq("0.00%");
    expect(Number(inflationInfo["round"]["ideal"])).to.eq(11132); // 5% / blocks per year * 10^9
    expect(inflationInfo.toHuman()["round"]["max"]).to.eq("0.00%");
    expect(Number(inflationInfo["round"]["max"])).to.eq(11132); // 5% / blocks per year * 10^9
  });
});

describeDevMoonbeam("Staking - Join Candidates", (context) => {
  const keyring = new Keyring({ type: "ethereum" });
  const ethan = keyring.addFromUri(ETHAN_PRIVKEY, null, "ethereum");

  before("should join as a candidate", async () => {
    await context.polkadotApi.tx.parachainStaking
      .joinCandidates(MIN_GLMR_STAKING, 1)
      .signAndSend(ethan);
    await context.createBlock();
  });

  afterEach("cleanup candidate leave request", async () => {
    let candidateCount = (await context.polkadotApi.query.parachainStaking.candidatePool()).length;
    await context.polkadotApi.tx.parachainStaking
      .cancelLeaveCandidates(candidateCount)
      .signAndSend(ethan);
    await context.createBlock();
  });

  it("should successfully call joinCandidates", async function () {
    let candidatesAfter = await context.polkadotApi.query.parachainStaking.candidatePool();
    expect(candidatesAfter.length).to.equal(2, "new candidate should have been added");
    expect(candidatesAfter[1].owner.toString()).to.equal(
      ETHAN,
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

    await context.polkadotApi.tx.parachainStaking
      .scheduleLeaveCandidates(candidatesAfter.length)
      .signAndSend(ethan);
    await context.createBlock();

    const candidateState = (
      await context.polkadotApi.query.parachainStaking.candidateInfo(ETHAN)
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

    await context.polkadotApi.tx.parachainStaking
      .scheduleLeaveCandidates(candidatesAfter.length)
      .signAndSend(ethan);
    await context.createBlock();

    const candidateState = (
      await context.polkadotApi.query.parachainStaking.candidateInfo(ETHAN)
    ).unwrap();
    expect(candidateState.status.isLeaving).to.be.true;

    const whenRound = candidateState.status.asLeaving.toNumber();
    await jumpToRound(context, whenRound - 1);

    await context.polkadotApi.tx.parachainStaking
      .executeLeaveCandidates(ETHAN, candidateState.delegationCount)
      .signAndSend(ethan);
    await context.createBlock();
    const extrinsicResult = await getExtrinsicResult(
      context,
      "parachainStaking",
      "executeLeaveCandidates"
    );
    expect(extrinsicResult).to.equal("CandidateCannotLeaveYet");

    await jumpToRound(context, whenRound);
    await context.polkadotApi.tx.parachainStaking
      .executeLeaveCandidates(ETHAN, candidateState.delegationCount)
      .signAndSend(ethan);
    await context.createBlock();
    const candidateStateAfter = await context.polkadotApi.query.parachainStaking.candidateInfo(
      ETHAN
    );
    expect(candidateStateAfter.isNone).to.be.true;
  });
});

describeDevMoonbeam("Staking - Join Delegators", (context) => {
  const keyring = new Keyring({ type: "ethereum" });
  const ethan = keyring.addFromUri(ETHAN_PRIVKEY, null, "ethereum");

  before("should successfully call delegate on ALITH", async function () {
    await context.polkadotApi.tx.parachainStaking
      .delegate(ALITH, MIN_GLMR_DELEGATOR, 0, 0)
      .signAndSend(ethan);
    await context.createBlock();
  });

  afterEach("cleanup delegator leave request", async () => {
    await context.polkadotApi.tx.parachainStaking.cancelLeaveDelegators().signAndSend(ethan);
    await context.createBlock();
  });

  it("should have successfully delegated stake to ALITH", async function () {
    const delegatorsAfter = (
      await context.polkadotApi.query.parachainStaking.delegatorState(ETHAN)
    ).unwrap();
    expect(delegatorsAfter.delegations[0].owner.toString()).to.equal(
      ALITH,
      "new delegation to alith should have been added"
    );
    expect(delegatorsAfter.delegations[0].amount.toBigInt()).to.equal(
      5n * GLMR,
      "delegation amount to alith should be 5"
    );
  });

  it("should successfully schedule leave delegators", async function () {
    await context.polkadotApi.tx.parachainStaking.scheduleLeaveDelegators().signAndSend(ethan);
    await context.createBlock();

    const delegatorState = (
      await context.polkadotApi.query.parachainStaking.delegatorState(ETHAN)
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
        (req) => req.delegator.eq(ETHAN) && req.action.isRevoke
      );
      expect(revokeRequest).to.not.be.undefined;
      expect(revokeRequest.whenExecutable.toNumber()).to.equal(currentRound + roundDelay);
    }
  });

  it("should successfully execute schedule leave delegators at correct round", async function () {
    this.timeout(20000);

    await context.polkadotApi.tx.parachainStaking.scheduleLeaveDelegators().signAndSend(ethan);
    await context.createBlock();

    const delegatorState = (
      await context.polkadotApi.query.parachainStaking.delegatorState(ETHAN)
    ).unwrap();
    const scheduledRequests =
      (await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(
        ALITH
      )) as unknown as any[];
    const revokeRequest = scheduledRequests.find(
      (req) => req.delegator.eq(ETHAN) && req.action.isRevoke
    );
    expect(revokeRequest).to.not.be.undefined;

    const whenRound = revokeRequest.whenExecutable.toNumber();
    await jumpToRound(context, whenRound - 1);

    await context.polkadotApi.tx.parachainStaking
      .executeLeaveDelegators(ETHAN, delegatorState.delegations.length)
      .signAndSend(ethan);
    await context.createBlock();
    const extrinsicResult = await getExtrinsicResult(
      context,
      "parachainStaking",
      "executeLeaveDelegators"
    );
    expect(extrinsicResult).to.equal("DelegatorCannotLeaveYet");

    await jumpToRound(context, whenRound);
    await context.polkadotApi.tx.parachainStaking
      .executeLeaveDelegators(ETHAN, delegatorState.delegations.length)
      .signAndSend(ethan);
    await context.createBlock();
    const delegatorStateAfter = await context.polkadotApi.query.parachainStaking.delegatorState(
      ETHAN
    );
    expect(delegatorStateAfter.isNone).to.be.true;
  });
});

describeDevMoonbeam("Staking - Delegation Requests", (context) => {
  const numberToHex = (n: BigInt): string => `0x${n.toString(16).padStart(32, "0")}`;

  const BOND_AMOUNT = MIN_GLMR_DELEGATOR + 100n;
  const BOND_AMOUNT_HEX = numberToHex(BOND_AMOUNT);

  let ethan: KeyringPair;
  let balathar: KeyringPair;
  beforeEach("should successfully call delegate on ALITH", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    ethan = keyring.addFromUri(ETHAN_PRIVKEY, null, "ethereum");
    balathar = keyring.addFromUri(BALTATHAR_PRIV_KEY, null, "ethereum");

    await context.polkadotApi.tx.parachainStaking
      .delegate(ALITH, BOND_AMOUNT, 0, 0)
      .signAndSend(ethan);
    await context.createBlock();
  });

  afterEach("should clean up delegation requests", async () => {
    await context.polkadotApi.tx.parachainStaking.cancelDelegationRequest(ALITH).signAndSend(ethan);
    await context.createBlock();
  });

  it("should successfully schedule revoke", async function () {
    const delegationRequestsBefore =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(ALITH);
    expect(delegationRequestsBefore.toJSON()).to.be.empty;

    const currentRound = (
      await context.polkadotApi.query.parachainStaking.round()
    ).current.toNumber();

    // schedule revoke
    await context.polkadotApi.tx.parachainStaking
      .scheduleRevokeDelegation(ALITH)
      .signAndSend(ethan);
    await context.createBlock();

    const delegationRequestsAfter =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(ALITH);
    const roundDelay = context.polkadotApi.consts.parachainStaking.revokeDelegationDelay.toNumber();
    expect(delegationRequestsAfter.toJSON()).to.deep.equal([
      {
        delegator: ETHAN,
        whenExecutable: currentRound + roundDelay,
        action: {
          revoke: BOND_AMOUNT_HEX,
        },
      },
    ]);
  });

  it("should successfully cancel revoke", async function () {
    const delegationRequestsBefore =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(ALITH);
    expect(delegationRequestsBefore.toJSON()).to.be.empty;

    const currentRound = (
      await context.polkadotApi.query.parachainStaking.round()
    ).current.toNumber();

    await context.polkadotApi.tx.parachainStaking
      .scheduleRevokeDelegation(ALITH)
      .signAndSend(ethan);
    await context.createBlock();
    const delegationRequestsAfterSchedule =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(ALITH);
    const roundDelay = context.polkadotApi.consts.parachainStaking.revokeDelegationDelay.toNumber();
    expect(delegationRequestsAfterSchedule.toJSON()).to.deep.equal([
      {
        delegator: ETHAN,
        whenExecutable: currentRound + roundDelay,
        action: {
          revoke: BOND_AMOUNT_HEX,
        },
      },
    ]);

    await context.polkadotApi.tx.parachainStaking.cancelDelegationRequest(ALITH).signAndSend(ethan);
    await context.createBlock();

    const delegationRequestsAfterCancel =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(ALITH);
    expect(delegationRequestsAfterCancel).to.be.empty;
  });

  it("should not execute revoke before target round", async function () {
    this.timeout(50000);

    const currentRound = (
      await context.polkadotApi.query.parachainStaking.round()
    ).current.toNumber();

    // schedule revoke
    await context.polkadotApi.tx.parachainStaking
      .scheduleRevokeDelegation(ALITH)
      .signAndSend(ethan);
    await context.createBlock();
    const delegationRequests =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(ALITH);
    expect(delegationRequests.toJSON()).to.not.be.empty;

    // jump to a round before the actual executable Round
    await jumpToRound(context, delegationRequests[0].whenExecutable - 1);

    // execute revoke
    await context.polkadotApi.tx.parachainStaking
      .executeDelegationRequest(ETHAN, ALITH)
      .signAndSend(ethan);

    await context.createBlock();
    const extrinsicResult = await getExtrinsicResult(
      context,
      "parachainStaking",
      "executeDelegationRequest"
    );
    expect(extrinsicResult).to.equal("PendingDelegationRequestNotDueYet");

    const { delegations: delegationsAfter } = (
      await context.polkadotApi.query.parachainStaking.delegatorState(ETHAN)
    ).unwrap();
    const delegationRequestsAfter =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(ALITH);

    expect(delegationsAfter.toJSON()).to.deep.equal([
      {
        owner: ALITH,
        amount: BOND_AMOUNT_HEX,
      },
    ]);
    const roundDelay = context.polkadotApi.consts.parachainStaking.revokeDelegationDelay.toNumber();
    expect(delegationRequestsAfter.toJSON()).to.deep.equal([
      {
        delegator: ETHAN,
        whenExecutable: currentRound + roundDelay,
        action: {
          revoke: BOND_AMOUNT_HEX,
        },
      },
    ]);
  });

  it("should successfully execute revoke", async function () {
    this.timeout(20000);

    // schedule revoke
    await context.polkadotApi.tx.parachainStaking
      .scheduleRevokeDelegation(ALITH)
      .signAndSend(ethan);
    await context.createBlock();
    const delegationRequests =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(ALITH);
    expect(delegationRequests.toJSON()).to.not.be.empty;

    // jump to executable Round
    await jumpToRound(context, delegationRequests[0].whenExecutable);

    // execute revoke
    await context.polkadotApi.tx.parachainStaking
      .executeDelegationRequest(ETHAN, ALITH)
      .signAndSend(ethan);
    await context.createBlock();

    const delegationsAfter = await context.polkadotApi.query.parachainStaking.delegatorState(ETHAN);
    const delegationRequestsAfter =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(ALITH);
    // last delegation revoked, so delegator marked as leaving
    expect(delegationsAfter.isNone).to.be.true;
    expect(delegationRequestsAfter.toJSON()).to.be.empty;
  });

  it("should successfully schedule bond less", async function () {
    const delegationRequestsBefore =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(ALITH);
    expect(delegationRequestsBefore.toJSON()).to.be.empty;

    const currentRound = (
      await context.polkadotApi.query.parachainStaking.round()
    ).current.toNumber();

    // schedule bond less
    await context.polkadotApi.tx.parachainStaking
      .scheduleDelegatorBondLess(ALITH, 10n)
      .signAndSend(ethan);
    await context.createBlock();

    const delegationRequestsAfter =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(ALITH);
    const roundDelay =
      context.polkadotApi.consts.parachainStaking.delegationBondLessDelay.toNumber();
    expect(delegationRequestsAfter.toJSON()).to.deep.equal([
      {
        delegator: ETHAN,
        whenExecutable: currentRound + roundDelay,
        action: {
          decrease: 10,
        },
      },
    ]);
  });

  it("should successfully cancel bond less", async function () {
    const delegationRequestsBefore =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(ALITH);
    expect(delegationRequestsBefore.toJSON()).to.be.empty;

    const currentRound = (
      await context.polkadotApi.query.parachainStaking.round()
    ).current.toNumber();

    const LESS_AMOUNT = 10;
    await context.polkadotApi.tx.parachainStaking
      .scheduleDelegatorBondLess(ALITH, LESS_AMOUNT)
      .signAndSend(ethan);
    await context.createBlock();
    const delegationRequestsAfterSchedule =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(ALITH);
    const roundDelay =
      context.polkadotApi.consts.parachainStaking.delegationBondLessDelay.toNumber();
    expect(delegationRequestsAfterSchedule.toJSON()).to.deep.equal([
      {
        delegator: ETHAN,
        whenExecutable: currentRound + roundDelay,
        action: {
          decrease: LESS_AMOUNT,
        },
      },
    ]);

    await context.polkadotApi.tx.parachainStaking.cancelDelegationRequest(ALITH).signAndSend(ethan);
    await context.createBlock();

    const delegationRequestsAfterCancel =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(ALITH);
    expect(delegationRequestsAfterCancel.toJSON()).to.be.empty;
  });

  it("should not execute bond less before target round", async function () {
    this.timeout(50000);

    const currentRound = (
      await context.polkadotApi.query.parachainStaking.round()
    ).current.toNumber();

    // schedule bond less
    const LESS_AMOUNT = 10;
    await context.polkadotApi.tx.parachainStaking
      .scheduleDelegatorBondLess(ALITH, LESS_AMOUNT)
      .signAndSend(ethan);
    await context.createBlock();
    const delegationRequests =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(ALITH);
    expect(delegationRequests.toJSON()).to.not.be.empty;

    // jump to a round before the actual executable Round
    await jumpToRound(context, delegationRequests[0].whenExecutable - 1);

    // execute bond less
    await context.polkadotApi.tx.parachainStaking
      .executeDelegationRequest(ETHAN, ALITH)
      .signAndSend(ethan);

    await context.createBlock();
    const extrinsicResult = await getExtrinsicResult(
      context,
      "parachainStaking",
      "executeDelegationRequest"
    );
    expect(extrinsicResult).to.equal("PendingDelegationRequestNotDueYet");

    const { delegations: delegationsAfter } = (
      await context.polkadotApi.query.parachainStaking.delegatorState(ETHAN)
    ).unwrap();
    const delegationRequestsAfter =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(ALITH);

    expect(delegationsAfter.toJSON()).to.deep.equal([
      {
        owner: ALITH,
        amount: BOND_AMOUNT_HEX,
      },
    ]);

    const roundDelay =
      context.polkadotApi.consts.parachainStaking.delegationBondLessDelay.toNumber();
    expect(delegationRequestsAfter.toJSON()).to.deep.equal([
      {
        delegator: ETHAN,
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
    await context.polkadotApi.tx.parachainStaking
      .scheduleDelegatorBondLess(ALITH, LESS_AMOUNT)
      .signAndSend(ethan);
    await context.createBlock();
    const delegationRequests =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(ALITH);
    expect(delegationRequests).to.not.be.empty;

    // jump to executable Round
    await jumpToRound(context, delegationRequests[0].whenExecutable);

    // execute bond less
    await context.polkadotApi.tx.parachainStaking
      .executeDelegationRequest(ETHAN, ALITH)
      .signAndSend(ethan);
    await context.createBlock();

    const {
      delegations: [firstDelegationAfter, ..._],
    } = (await context.polkadotApi.query.parachainStaking.delegatorState(ETHAN)).unwrap();
    const delegationRequestsAfter =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(ALITH);
    expect(firstDelegationAfter.toJSON()).to.deep.equal({
      owner: ALITH,
      amount: numberToHex(BOND_AMOUNT - BigInt(LESS_AMOUNT)),
    });
    expect(delegationRequestsAfter.toJSON()).to.be.empty;
  });

  it("should successfully remove scheduled requests on collator leave", async function () {
    this.timeout(20000);

    await context.polkadotApi.tx.parachainStaking
      .joinCandidates(100n * BOND_AMOUNT, 1)
      .signAndSend(balathar);
    await context.createBlock();

    await context.polkadotApi.tx.parachainStaking
      .delegate(BALTATHAR, BOND_AMOUNT, 0, 1)
      .signAndSend(ethan);
    await context.createBlock();

    const delegationRequestsBefore =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(ALITH);
    expect(delegationRequestsBefore.toJSON()).to.be.empty;

    // schedule bond less
    await context.polkadotApi.tx.parachainStaking
      .scheduleDelegatorBondLess(BALTATHAR, 10n)
      .signAndSend(ethan);
    await context.createBlock();
    await context.polkadotApi.tx.parachainStaking.scheduleLeaveCandidates(2).signAndSend(balathar);
    await context.createBlock();

    const collatorState = await context.polkadotApi.query.parachainStaking.candidateInfo(BALTATHAR);
    await jumpToRound(context, collatorState.unwrap().status.asLeaving.toNumber());

    await context.polkadotApi.tx.parachainStaking
      .executeLeaveCandidates(BALTATHAR, 1)
      .signAndSend(ethan);
    await context.createBlock();
    const delegationRequestsAfter =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(ALITH);
    expect(delegationRequestsAfter.toJSON()).to.be.empty;
  });

  it("should successfully remove scheduled requests on delegator leave", async function () {
    this.timeout(20000);

    const delegationRequestsBefore =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(ALITH);
    expect(delegationRequestsBefore.toJSON()).to.be.empty;

    // schedule bond less
    await context.polkadotApi.tx.parachainStaking
      .scheduleDelegatorBondLess(ALITH, 10n)
      .signAndSend(ethan);
    await context.createBlock();
    await context.polkadotApi.tx.parachainStaking.scheduleLeaveDelegators().signAndSend(ethan);
    await context.createBlock();

    const delegatorState = await context.polkadotApi.query.parachainStaking.delegatorState(ETHAN);
    const scheduledRequests =
      (await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(
        ALITH
      )) as unknown as any[];
    const revokeRequest = scheduledRequests.find(
      (req) => req.delegator.eq(ETHAN) && req.action.isRevoke
    );
    expect(revokeRequest).to.not.be.undefined;
    await jumpToRound(context, revokeRequest.whenExecutable.toNumber());

    await context.polkadotApi.tx.parachainStaking
      .executeLeaveDelegators(ETHAN, 1)
      .signAndSend(ethan);
    const block = await context.createBlock();
    const delegationRequestsAfter =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(ALITH);
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
  const BOND_AMOUNT = MIN_GLMR_NOMINATOR + 100n;
  const keyring = new MIN_GLMR_DELEGATORthereum" });
  const ethan = keyring.addFromUri(ETHAN_PRIVKEY, null, "ethereum");

  before("should successfully call delegate on ALITH", async () => {
    await context.polkadotApi.tx.parachainStaking
      .delegate(ALITH, BOND_AMOUNT, 0, 0)
      .signAndSend(ethan);
    await context.createBlock();
  });

  afterEach("should clean up delegation requests", async () => {
    await context.polkadotApi.tx.parachainStaking.cancelDelegationRequest(ALITH).signAndSend(ethan);
    await context.createBlock();
  });

  it("should allow bond more when no delgation request scheduled", async function () {
    const bondAmountBefore = (
      await context.polkadotApi.query.parachainStaking.delegatorState(ETHAN)
    ).unwrap().total;

    // schedule bond less
    const increaseAmount = 5;
    await context.polkadotApi.tx.parachainStaking
      .delegatorBondMore(ALITH, increaseAmount)
      .signAndSend(ethan);
    await context.createBlock();

    const bondAmountAfter = (
      await context.polkadotApi.query.parachainStaking.delegatorState(ETHAN)
    ).unwrap().total;
    expect(bondAmountAfter.eq(bondAmountBefore.addn(increaseAmount))).to.be.true;
  });

  it("should allow bond more when bond less schedule", async function () {
    const bondAmountBefore = (
      await context.polkadotApi.query.parachainStaking.delegatorState(ETHAN)
    ).unwrap().total;

    // schedule bond less
    await context.polkadotApi.tx.parachainStaking
      .scheduleDelegatorBondLess(ALITH, 10n)
      .signAndSend(ethan);
    await context.createBlock();

    const increaseAmount = 5;
    await context.polkadotApi.tx.parachainStaking
      .delegatorBondMore(ALITH, increaseAmount)
      .signAndSend(ethan);
    await context.createBlock();

    const bondAmountAfter = (
      await context.polkadotApi.query.parachainStaking.delegatorState(ETHAN)
    ).unwrap().total;
    expect(bondAmountAfter.eq(bondAmountBefore.addn(increaseAmount))).to.be.true;
  });

  it("should not allow bond more when revoke schedule", async function () {
    const bondAmountBefore = (
      await context.polkadotApi.query.parachainStaking.delegatorState(ETHAN)
    ).unwrap().total;

    // schedule bond less
    await context.polkadotApi.tx.parachainStaking
      .scheduleRevokeDelegation(ALITH)
      .signAndSend(ethan);
    await context.createBlock();

    const increaseAmount = 5n;
    await context.polkadotApi.tx.parachainStaking
      .delegatorBondMore(ALITH, increaseAmount)
      .signAndSend(ethan);
    await context.createBlock();

    const extrinsicError = await getExtrinsicResult(
      context,
      "parachainStaking",
      "delegatorBondMore"
    );
    expect(extrinsicError).to.equal("PendingDelegationRevoke");
    const bondAmountAfter = (
      await context.polkadotApi.query.parachainStaking.delegatorState(ETHAN)
    ).unwrap().total;
    expect(bondAmountAfter.eq(bondAmountBefore)).to.be.true;
  });
});

describeDevMoonbeam("Staking - Rewards", (context) => {
  const EXTRA_BOND_AMOUNT = 1_000_000_000_000_000_000n;
  const BOND_AMOUNT = MIN_GLMR_NOMINATOR + EXTRA_BOND_AMOUNT;

  let ethan: KeyringPair;
  let balathar: KeyringPair;
  beforeEach("should successfully call delegate on ALITH", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    ethan = keyring.addFromUri(ETHAN_PRIVKEY, null, "ethereum");
    balathar = keyring.addFromUri(BALTATHAR_PRIV_KEY, null, "ethereum");

    await context.polkadotApi.tx.parachainStaking
      .delegate(ALITH, BOND_AMOUNT, 0, 0)
      .signAndSend(ethan);
    await context.createBlock();
    const currentRound = await context.polkadotApi.query.parachainStaking.round();
    await jumpToRound(context, currentRound.current.addn(1).toNumber());
  });

  afterEach("should clean up delegation requests", async () => {
    await context.polkadotApi.tx.parachainStaking.cancelDelegationRequest(ALITH).signAndSend(ethan);
    await context.createBlock();
    await context.polkadotApi.tx.parachainStaking.cancelLeaveDelegators().signAndSend(ethan);
    await context.createBlock();
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

    await context.polkadotApi.tx.parachainStaking.scheduleLeaveDelegators().signAndSend(ethan);
    await context.createBlock();

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

    await context.polkadotApi.tx.parachainStaking
      .scheduleRevokeDelegation(ALITH)
      .signAndSend(ethan);
    await context.createBlock();

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

    await context.polkadotApi.tx.parachainStaking
      .delegate(ALITH, BOND_AMOUNT, 1, 0)
      .signAndSend(balathar);
    await context.polkadotApi.tx.parachainStaking
      .scheduleDelegatorBondLess(ALITH, EXTRA_BOND_AMOUNT)
      .signAndSend(ethan);
    await context.createBlock();

    const rewardDelay = context.polkadotApi.consts.parachainStaking.rewardPaymentDelay;
    const currentRound = (await context.polkadotApi.query.parachainStaking.round()).current;
    const blockHash = await jumpToRound(context, currentRound.add(rewardDelay).addn(1).toNumber());
    let rewardedEvents = await getRewardedEventsAt(context, blockHash);
    let rewardedEthan = rewardedEvents.find(({ account }) => account == ethan.address);
    let rewardedBalathar = rewardedEvents.find(({ account }) => account == balathar.address);
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
        amount: event.event.data[1],
      });
    }
  }

  return rewardedEvents;
}

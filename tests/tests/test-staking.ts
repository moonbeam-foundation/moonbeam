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
  MIN_GLMR_NOMINATOR,
  GLMR,
} from "../util/constants";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { KeyringPair } from "@substrate/txwrapper-core";
import { Option } from "@polkadot/types-codec";

describeDevMoonbeam("Staking - Genesis", (context) => {
  it("should match collator reserved bond reserved", async function () {
    const account = (await context.polkadotApi.query.system.account(COLLATOR_ACCOUNT)) as any;
    const expectedReserved = DEFAULT_GENESIS_STAKING + DEFAULT_GENESIS_MAPPING;
    expect(account.data.reserved.toString()).to.equal(expectedReserved.toString());
  });

  it("should include collator from the specs", async function () {
    const collators = await context.polkadotApi.query.parachainStaking.selectedCandidates();
    expect((collators[0] as Buffer).toString("hex")).equal(COLLATOR_ACCOUNT);
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
  it("should successfully call joinCandidates on ETHAN", async function () {
    const keyring = new Keyring({ type: "ethereum" });
    const ethan = await keyring.addFromUri(ETHAN_PRIVKEY, null, "ethereum");
    await context.polkadotApi.tx.parachainStaking
      .joinCandidates(MIN_GLMR_STAKING, 1)
      .signAndSend(ethan);
    await context.createBlock();

    let candidatesAfter = (await context.polkadotApi.query.parachainStaking.candidatePool()) as any;
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
});

describeDevMoonbeam("Staking - Join Delegators", (context) => {
  let ethan;
  before("should successfully call delegate on ALITH", async function () {
    const keyring = new Keyring({ type: "ethereum" });
    ethan = await keyring.addFromUri(ETHAN_PRIVKEY, null, "ethereum");
    await context.polkadotApi.tx.parachainStaking
      .delegate(ALITH, MIN_GLMR_NOMINATOR, 0, 0)
      .signAndSend(ethan);
    await context.createBlock();
  });
  it("should have successfully delegated stake to ALITH", async function () {
    const delegatorsAfter = (
      (await context.polkadotApi.query.parachainStaking.delegatorState(ETHAN)) as any
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
});

describeDevMoonbeam("Staking - Delegation Requests", (context) => {
  const numberToHex = (n: BigInt): string => `0x${n.toString(16).padStart(32, "0")}`;

  const BOND_AMOUNT = MIN_GLMR_NOMINATOR + 100n;
  const BOND_AMOUNT_HEX = numberToHex(BOND_AMOUNT);

  let ethan: KeyringPair;
  beforeEach("should successfully call delegate on ALITH", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    ethan = keyring.addFromUri(ETHAN_PRIVKEY, null, "ethereum");
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
    const delegationRequestsBefore = (
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(ALITH)
    ).toJSON();
    expect(delegationRequestsBefore).to.be.empty;

    const currentRound = (
      (await context.polkadotApi.query.parachainStaking.round()).toJSON() as any
    ).current;

    // schedule revoke
    await context.polkadotApi.tx.parachainStaking
      .scheduleRevokeDelegation(ALITH)
      .signAndSend(ethan);
    await context.createBlock();

    const delegationRequestsAfter = (
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(ALITH)
    ).toJSON();
    expect(delegationRequestsAfter).to.deep.equal([
      {
        delegator: ETHAN,
        whenExecutable: currentRound + 2,
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
    const delegationRequest = (
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(ALITH)
    ).toJSON();
    expect(delegationRequest).to.not.be.empty;

    // jump to executable Round
    const whenExecutable = delegationRequest[0].whenExecutable;
    while (true) {
      const currentRound = (
        (await context.polkadotApi.query.parachainStaking.round()).toJSON() as any
      ).current;
      if (currentRound > whenExecutable) {
        break;
      }
      await context.createBlock();
    }

    // execute revoke
    await context.polkadotApi.tx.parachainStaking
      .executeDelegationRequest(ETHAN, ALITH)
      .signAndSend(ethan);
    await context.createBlock();

    const delegationsAfter = (await context.polkadotApi.query.parachainStaking.delegatorState(
      ETHAN
    )) as Option<any>;
    const delegationRequestsAfter = (
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(ALITH)
    ).toJSON();
    // last delegation revoked, so delegator marked as leaving
    expect(delegationsAfter.isNone).to.be.true;
    expect(delegationRequestsAfter).to.be.empty;
  });

  it("should successfully schedule bond less", async function () {
    const delegationRequestsBefore = (
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(ALITH)
    ).toJSON();
    expect(delegationRequestsBefore).to.be.empty;

    const currentRound = (
      (await context.polkadotApi.query.parachainStaking.round()).toJSON() as any
    ).current;

    // schedule bond less
    await context.polkadotApi.tx.parachainStaking
      .scheduleDelegatorBondLess(ALITH, 10n)
      .signAndSend(ethan);
    await context.createBlock();

    const delegationRequestsAfter = (
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(ALITH)
    ).toJSON();
    expect(delegationRequestsAfter).to.deep.equal([
      {
        delegator: ETHAN,
        whenExecutable: currentRound + 2,
        action: {
          decrease: 10,
        },
      },
    ]);
  });

  it("should successfully execute bond less", async function () {
    this.timeout(20000);

    const LESS_AMOUNT = 10n;

    // schedule bond less
    await context.polkadotApi.tx.parachainStaking
      .scheduleDelegatorBondLess(ALITH, LESS_AMOUNT)
      .signAndSend(ethan);
    await context.createBlock();
    const delegationRequest = (
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(ALITH)
    ).toJSON();
    expect(delegationRequest).to.not.be.empty;

    // jump to executable Round
    const whenExecutable = delegationRequest[0].whenExecutable;
    while (true) {
      const currentRound = (
        (await context.polkadotApi.query.parachainStaking.round()).toJSON() as any
      ).current;
      if (currentRound > whenExecutable) {
        break;
      }
      await context.createBlock();
    }

    // execute bond less
    await context.polkadotApi.tx.parachainStaking
      .executeDelegationRequest(ETHAN, ALITH)
      .signAndSend(ethan);
    await context.createBlock();

    const delegationsAfter = (
      (await context.polkadotApi.query.parachainStaking.delegatorState(ETHAN)) as any
    )
      .unwrap()
      .delegations.toJSON();
    const delegationRequestsAfter = (
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(ALITH)
    ).toJSON();
    expect(delegationsAfter[0]).to.deep.equal({
      owner: ALITH,
      amount: numberToHex(BOND_AMOUNT - LESS_AMOUNT),
    });
    expect(delegationRequestsAfter).to.be.empty;
  });
});

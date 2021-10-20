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
import { createBlockWithExtrinsic } from "../util/substrate-rpc";

describeDevMoonbeam("Staking - Genesis", (context) => {
  it("should match collator reserved bond reserved", async function () {
    const account = await context.polkadotApi.query.system.account(COLLATOR_ACCOUNT);
    const expectedReserved = DEFAULT_GENESIS_STAKING + DEFAULT_GENESIS_MAPPING;
    expect(account.data.reserved.toString()).to.equal(expectedReserved.toString());
  });

  it("should include collator from the specs", async function () {
    const collators = await context.polkadotApi.query.parachainStaking.selectedCandidates();
    expect((collators[0] as Buffer).toString("hex").toLowerCase()).equal(COLLATOR_ACCOUNT);
  });

  it("should have collator state as defined in the specs", async function () {
    const collator = await context.polkadotApi.query.parachainStaking.candidateState(
      COLLATOR_ACCOUNT
    );
    expect(collator.toHuman()["id"].toLowerCase()).equal(COLLATOR_ACCOUNT);
    expect(collator.toHuman()["state"]).equal("Active");
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
    expect(inflationInfo.toHuman()["expect"]["min"]).to.eq("100.0000 kUNIT");
    expect(inflationInfo.toHuman()["expect"]["ideal"]).to.eq("200.0000 kUNIT");
    expect(inflationInfo.toHuman()["expect"]["max"]).to.eq("500.0000 kUNIT");
    expect(inflationInfo.toHuman()["annual"]["min"]).to.eq("4.00%");
    expect(inflationInfo.toHuman()["annual"]["ideal"]).to.eq("5.00%");
    expect(inflationInfo.toHuman()["annual"]["max"]).to.eq("5.00%");
    expect(inflationInfo.toHuman()["round"]["min"]).to.eq("0.00%");
    expect(Number(inflationInfo["round"]["min"])).to.eq(4563); // 4% / 8766 * 10^9
    expect(inflationInfo.toHuman()["round"]["ideal"]).to.eq("0.00%");
    expect(Number(inflationInfo["round"]["ideal"])).to.eq(5703); // 5% / 8766 * 10^9
    expect(inflationInfo.toHuman()["round"]["max"]).to.eq("0.00%");
    expect(Number(inflationInfo["round"]["max"])).to.eq(5703); // 5% / 8766 * 10^9
  });
});

describeDevMoonbeam("Staking - Join Candidates", (context) => {
  it("should succesfully call joinCandidates on ETHAN", async function () {
    const keyring = new Keyring({ type: "ethereum" });
    const ethan = await keyring.addFromUri(ETHAN_PRIVKEY, null, "ethereum");
    await context.polkadotApi.tx.parachainStaking
      .joinCandidates(MIN_GLMR_STAKING, 1)
      .signAndSend(ethan);
    await context.createBlock();

    let candidatesAfter = await context.polkadotApi.query.parachainStaking.candidatePool();
    expect(
      (candidatesAfter.toHuman() as { owner: string; amount: string }[]).length === 2
    ).to.equal(true, "new candidate should have been added");
    expect(
      (candidatesAfter.toHuman() as { owner: string; amount: string }[])[1].owner === ETHAN
    ).to.equal(true, "new candidate ethan should have been added");
    expect(
      (candidatesAfter.toHuman() as { owner: string; amount: string }[])[1].amount ===
        "1.0000 kUNIT"
    ).to.equal(true, "new candidate ethan should have been added (wrong amount)");
  });
});

describeDevMoonbeam("Staking - Join Delegators", (context) => {
  let ethan;
  before("should succesfully call delegate on ALITH", async function () {
    const keyring = new Keyring({ type: "ethereum" });
    ethan = await keyring.addFromUri(ETHAN_PRIVKEY, null, "ethereum");
    await context.polkadotApi.tx.parachainStaking
      .delegate(ALITH, MIN_GLMR_NOMINATOR, 0, 0)
      .signAndSend(ethan);
    await context.createBlock();
  });
  it("should have succesfully called delegate on ALITH", async function () {
    const delegatorsAfter = await context.polkadotApi.query.parachainStaking.delegatorState(ETHAN);
    expect(
      (
        delegatorsAfter.toHuman() as {
          delegations: { owner: string; amount: string }[];
        }
      ).delegations[0].owner === ALITH
    ).to.equal(true, "delegation didn't go through");
    expect(delegatorsAfter.toHuman()["status"]).equal("Active");
    expect(delegatorsAfter.toHuman()["delegations"][0].owner).equal(ALITH);
    expect(delegatorsAfter.toHuman()["delegations"][0].amount).equal("5.0000 UNIT");
  });
});

describeDevMoonbeam("Staking - Delegators cannot bond less than minimum delegation", (context) => {
  let ethan;
  before("should succesfully call delegate on ALITH", async function () {
    const keyring = new Keyring({ type: "ethereum" });
    ethan = await keyring.addFromUri(ETHAN_PRIVKEY, null, "ethereum");
    // Delegate
    await context.polkadotApi.tx.parachainStaking
      .delegate(ALITH, MIN_GLMR_NOMINATOR, 0, 0)
      .signAndSend(ethan);
    await context.createBlock();
  });
  it("should fail calling delegatorBondLess under min delegation amount", async function () {
    const { events } = await createBlockWithExtrinsic(
      context,
      ethan,
      context.polkadotApi.tx.parachainStaking.delegatorBondLess(ALITH, 1n * GLMR)
    );
    expect(events[1].toHuman().method).to.eq("ExtrinsicFailed");
    const delegatorsAfter = await context.polkadotApi.query.parachainStaking.delegatorState(ETHAN);
    expect(
      (
        delegatorsAfter.toHuman() as {
          delegations: { owner: string; amount: string }[];
        }
      ).delegations[0].owner === ALITH
    ).to.equal(true, "delegation didn't go through");
    expect(delegatorsAfter.toHuman()["delegations"][0].amount).equal("5.0000 UNIT");
  });
});

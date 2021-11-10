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
  MIN_GLMR_NOMINATOR_PLUS_ONE,
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
    const collator = await context.polkadotApi.query.parachainStaking.collatorState2(
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
    expect(inflationInfo["expect"]["min"].toBigInt()).to.eq(100_000n * GLMR);
    expect(inflationInfo["expect"]["ideal"].toBigInt()).to.eq(200_000n * GLMR);
    expect(inflationInfo["expect"]["max"].toBigInt()).to.eq(500_000n * GLMR);
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

    let candidatesAfter = (await context.polkadotApi.query.parachainStaking.candidatePool()) as any;
    expect(candidatesAfter.length).to.equal(2, "new candidate should have been added");
    expect(candidatesAfter[1].owner.toHex()).to.equal(
      ETHAN.toLowerCase(),
      "new candidate ethan should have been added"
    );
    expect(candidatesAfter[1].amount.toBigInt()).to.equal(
      1000n * GLMR,
      "new candidate ethan should have been added (wrong amount)"
    );
  });
});

describeDevMoonbeam("Staking - Candidate bond more", (context) => {
  let ethan;

  before("should succesfully call joinCandidates on ETHAN", async function () {
    const keyring = new Keyring({ type: "ethereum" });
    ethan = await keyring.addFromUri(ETHAN_PRIVKEY, null, "ethereum");
    await context.polkadotApi.tx.parachainStaking
      .joinCandidates(MIN_GLMR_STAKING, 1)
      .signAndSend(ethan);
    await context.createBlock();
  });

  it("should succesfully call candidateBondMore on ETHAN", async function () {
    await context.polkadotApi.tx.parachainStaking
      .candidateBondMore(MIN_GLMR_STAKING)
      .signAndSend(ethan);
    await context.createBlock();
    let candidatesAfter = await context.polkadotApi.query.parachainStaking.candidatePool();
    expect(candidatesAfter[1].amount.toBigInt()).to.equal(
      2000n * GLMR,
      "bond should have increased"
    );
  });
});

describeDevMoonbeam("Staking - Candidate bond less", (context) => {
  let ethan;

  before("call joinCandidates on ETHAN", async function () {
    const keyring = new Keyring({ type: "ethereum" });
    ethan = await keyring.addFromUri(ETHAN_PRIVKEY, null, "ethereum");
    await context.polkadotApi.tx.parachainStaking
      .joinCandidates(MIN_GLMR_STAKING, 1)
      .signAndSend(ethan);
    await context.createBlock();
    // add more stake
    await context.polkadotApi.tx.parachainStaking
      .candidateBondMore(MIN_GLMR_STAKING)
      .signAndSend(ethan);
    await context.createBlock();
    let candidatesAfter = await context.polkadotApi.query.parachainStaking.candidatePool();
    expect(candidatesAfter[1].amount.toBigInt()).to.equal(
      2000n * GLMR,
      "bond should have decreased"
    );
  });

  it("should succesfully call candidateBondLess on ETHAN", async function () {
    const { events } = await createBlockWithExtrinsic(
      context,
      ethan,
      context.polkadotApi.tx.parachainStaking.candidateBondLess(MIN_GLMR_STAKING)
    );
    expect(events[5].toHuman().method).to.eq("ExtrinsicSuccess");
    let candidatesAfter = await context.polkadotApi.query.parachainStaking.candidatePool();
    expect(candidatesAfter[1].amount.toBigInt()).to.equal(
      1000n * GLMR,
      "bond should have decreased"
    );
  });
});

describeDevMoonbeam("Staking - Candidate bond less", (context) => {
  let ethan;

  before("should succesfully call joinCandidates on ETHAN", async function () {
    const keyring = new Keyring({ type: "ethereum" });
    ethan = await keyring.addFromUri(ETHAN_PRIVKEY, null, "ethereum");
    await context.polkadotApi.tx.parachainStaking
      .joinCandidates(MIN_GLMR_STAKING, 1)
      .signAndSend(ethan);
    await context.createBlock();
    let candidatesAfter = await context.polkadotApi.query.parachainStaking.candidatePool();
    // TODO: Remove expect from before, to move in its own test
    expect(candidatesAfter[1].amount.toBigInt()).to.equal(
      1000n * GLMR,
      "bond should have decreased"
    );
  });

  it("should fail to call candidateBondLess on ETHAN below minimum amount", async function () {
    const { events } = await createBlockWithExtrinsic(
      context,
      ethan,
      context.polkadotApi.tx.parachainStaking.candidateBondLess(MIN_GLMR_NOMINATOR)
    );
    expect(events[3].toHuman().method).to.eq("ExtrinsicFailed");
    let candidatesAfter = await context.polkadotApi.query.parachainStaking.candidatePool();
    expect(candidatesAfter[1].amount.toBigInt()).to.equal(
      1000n * GLMR,
      "bond should have decreased"
    );
  });
});

describeDevMoonbeam("Staking - Join Nominators", (context) => {
  let ethan;

  beforeEach("should succesfully call nominate on ALITH", async function () {
    const keyring = new Keyring({ type: "ethereum" });
    ethan = await keyring.addFromUri(ETHAN_PRIVKEY, null, "ethereum");
    await context.polkadotApi.tx.parachainStaking
      .nominate(ALITH, MIN_GLMR_NOMINATOR, 0, 0)
      .signAndSend(ethan);
    await context.createBlock();
  });

  it("should have succesfully called nominate on ALITH", async function () {
    const nominatorsAfter = (
      (await context.polkadotApi.query.parachainStaking.nominatorState2(ETHAN)) as any
    ).unwrap();
    expect(nominatorsAfter.nominations[0].owner.toHex()).to.equal(
      ALITH.toLowerCase(),
      "nomination didnt go through"
    );
    expect(nominatorsAfter.status.toString()).equal("Active");
    expect(nominatorsAfter.nominations[0].owner.toHex()).equal(ALITH.toLowerCase());
    expect(nominatorsAfter.nominations[0].amount.toBigInt()).equal(5n * GLMR);
  });

  it("should succesfully revoke nomination on ALITH", async function () {
    await context.polkadotApi.tx.parachainStaking.revokeNomination(ALITH).signAndSend(ethan);
    await context.createBlock();

    const nominatorsAfter = await context.polkadotApi.query.parachainStaking.nominatorState2(ETHAN);
    expect(nominatorsAfter.toHuman()["status"].Leaving).equal("3");
  });
});

describeDevMoonbeam("Staking - Nominators Bond More", (context) => {
  let ethan;

  before("should succesfully call nominate on ALITH", async function () {
    const keyring = new Keyring({ type: "ethereum" });
    ethan = await keyring.addFromUri(ETHAN_PRIVKEY, null, "ethereum");
    // Nominate
    await context.polkadotApi.tx.parachainStaking
      .nominate(ALITH, MIN_GLMR_NOMINATOR, 0, 0)
      .signAndSend(ethan);
    await context.createBlock();
    // Bond More
    await context.polkadotApi.tx.parachainStaking
      .nominatorBondMore(ALITH, MIN_GLMR_NOMINATOR_PLUS_ONE)
      .signAndSend(ethan);
    await context.createBlock();
  });

  it("should succesfully call nominatorBondMore on ALITH", async function () {
    const nominatorsAfter = (
      (await context.polkadotApi.query.parachainStaking.nominatorState2(ETHAN)) as any
    ).unwrap();
    expect(nominatorsAfter.nominations[0].owner.toString()).to.equal(
      ALITH.toLowerCase(),
      "nomination didnt go through"
    );
    expect(nominatorsAfter.nominations[0].amount.toBigInt()).equal(11n * GLMR);
  });

  it("should succesfully call nominatorBondLess on ALITH", async function () {
    const { events } = await createBlockWithExtrinsic(
      context,
      ethan,
      context.polkadotApi.tx.parachainStaking.nominatorBondLess(ALITH, MIN_GLMR_NOMINATOR)
    );
    expect(events[2].method.toString()).to.eq("NominationDecreased");
    expect((events[2].data[2] as any).toBigInt()).to.eq(5n * GLMR);
    const nominatorsAfter = (
      (await context.polkadotApi.query.parachainStaking.nominatorState2(ETHAN)) as any
    ).unwrap();
    expect(nominatorsAfter.nominations[0].owner.toString()).to.equal(
      ALITH.toLowerCase(),
      "nomination didnt go through"
    );
    expect(nominatorsAfter["nominations"][0].amount.toBigInt()).equal(6n * GLMR);
  });
});

describeDevMoonbeam("Staking - Nominators shouldn't bond less than min bond", (context) => {
  let ethan;

  before("should succesfully call nominate on ALITH", async function () {
    const keyring = new Keyring({ type: "ethereum" });
    ethan = await keyring.addFromUri(ETHAN_PRIVKEY, null, "ethereum");
    // Nominate
    await context.polkadotApi.tx.parachainStaking
      .nominate(ALITH, MIN_GLMR_NOMINATOR, 0, 0)
      .signAndSend(ethan);
    await context.createBlock();
    // Bond More
    await context.polkadotApi.tx.parachainStaking
      .nominatorBondMore(ALITH, MIN_GLMR_NOMINATOR)
      .signAndSend(ethan);
    await context.createBlock();
  });

  it("should fail calling nominatorBondLess under min nomination amount", async function () {
    const { events } = await createBlockWithExtrinsic(
      context,
      ethan,
      context.polkadotApi.tx.parachainStaking.nominatorBondLess(ALITH, MIN_GLMR_NOMINATOR_PLUS_ONE)
    );
    expect(events[3].method.toString()).to.eq("ExtrinsicFailed");
    const nominatorsAfter = (
      (await context.polkadotApi.query.parachainStaking.nominatorState2(ETHAN)) as any
    ).unwrap();
    expect(nominatorsAfter.nominations[0].owner.toString()).to.equal(
      ALITH.toLowerCase(),
      "nomination didnt go through"
    );
    expect(nominatorsAfter.nominations[0].amount.toBigInt()).equal(10n * GLMR);
  });
});

describeDevMoonbeam(
  "Staking - Nominators shouldn't bond less than min bond - only bond less",
  (context) => {
    let ethan;

    before("should succesfully call nominate on ALITH", async function () {
      const keyring = new Keyring({ type: "ethereum" });
      ethan = await keyring.addFromUri(ETHAN_PRIVKEY, null, "ethereum");
      // Nominate
      await context.polkadotApi.tx.parachainStaking
        .nominate(ALITH, MIN_GLMR_NOMINATOR, 0, 0)
        .signAndSend(ethan);
      await context.createBlock();
    });

    it("should fail calling nominatorBondLess under min nomination amount ABCD", async function () {
      const { events } = await createBlockWithExtrinsic(
        context,
        ethan,
        context.polkadotApi.tx.parachainStaking.nominatorBondLess(ALITH, 1n * GLMR)
      );
      expect(events[3].method.toString()).to.eq("ExtrinsicFailed");
      const nominatorsAfter = (
        (await context.polkadotApi.query.parachainStaking.nominatorState2(ETHAN)) as any
      ).unwrap();
      expect(nominatorsAfter.nominations[0].owner.toString()).to.equal(
        ALITH.toLowerCase(),
        "nomination didnt go through"
      );
      expect(nominatorsAfter.nominations[0].amount.toBigInt()).equal(5n * GLMR);
    });
  }
);

// // TODO: bring back when we figure out how to get `NominatorState2.revocations`
// describeDevMoonbeam("Staking - Revoke Nomination", (context) => {
//   let ethan;
//   before("should succesfully call nominate on ALITH", async function () {
//     //nominate
//     const keyring = new Keyring({ type: "ethereum" });
//     ethan = await keyring.addFromUri(ETHAN_PRIVKEY, null, "ethereum");
//     await context.polkadotApi.tx.parachainStaking
//       .nominate(ALITH, MIN_GLMR_NOMINATOR, 0, 0)
//       .signAndSend(ethan);
//     await context.createBlock();
//   });
//   it("should succesfully revoke nomination for ALITH", async function () {
//     await context.polkadotApi.tx.parachainStaking.revokeNomination(ALITH).signAndSend(ethan);
//     await context.createBlock();
//     const nominatorsAfterRevocation =
//       await context.polkadotApi.query.parachainStaking.nominatorState2(ETHAN);
//     expect(
//       (nominatorsAfterRevocation.revocations[0] === ALITH).to.equal(
//         true,
//         "revocation didnt go through"
//       )
//     );
//   });
// });

import "@polkadot/api-augment";
import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { MIN_GLMR_STAKING, MIN_GLMR_DELEGATOR } from "../../util/constants";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { alith, baltathar, charleth, ethan } from "../../util/accounts";
import { expectOk } from "../../util/expect";

describeDevMoonbeam("Staking - Delegator Join - bond less than min", (context) => {
  it("should fail", async () => {
    const minDelegatorStk = context.polkadotApi.consts.parachainStaking.minDelegation;
    const block = await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegate(alith.address, minDelegatorStk.subn(10), 0, 0)
        .signAsync(ethan)
    );
    expect(block.result.successful).to.be.false;
    expect(block.result.error.name).to.equal("DelegationBelowMin");
  });
});

describeDevMoonbeam("Staking - Delegator Join- candidate not exists", (context) => {
  it("should fail", async () => {
    const block = await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegate(baltathar.address, MIN_GLMR_DELEGATOR, 0, 0)
        .signAsync(ethan)
    );
    expect(block.result.successful).to.be.false;
    expect(block.result.error.name).to.equal("CandidateDNE");
  });
});

describeDevMoonbeam("Staking - Delegator Join- candidate not exists and self", (context) => {
  it("should fail", async () => {
    const block = await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegate(ethan.address, MIN_GLMR_DELEGATOR, 0, 0)
        .signAsync(ethan)
    );
    expect(block.result.successful).to.be.false;
    expect(block.result.error.name).to.equal("CandidateDNE");
  });
});

describeDevMoonbeam("Staking - Delegator Join - already a candidate", (context) => {
  it("should fail", async () => {
    const block = await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegate(alith.address, MIN_GLMR_DELEGATOR, 0, 0)
        .signAsync(alith)
    );
    expect(block.result.successful).to.be.false;
    expect(block.result.error.name).to.equal("CandidateExists");
  });
});

describeDevMoonbeam("Staking - Delegator Join - already delegated", (context) => {
  before("should delegate", async () => {
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .delegate(alith.address, MIN_GLMR_DELEGATOR, 0, 0)
          .signAsync(ethan)
      )
    );
  });

  it("should fail", async () => {
    const block = await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegate(alith.address, MIN_GLMR_DELEGATOR, 0, 1)
        .signAsync(ethan)
    );
    expect(block.result.successful).to.be.false;
    expect(block.result.error.name).to.equal("AlreadyDelegatedCandidate");
  });
});

describeDevMoonbeam("Staking - Delegator Join - wrong candidate delegation hint", (context) => {
  before(
    "should setup alith+baltathar are candidates and ethan+chaleth as delegators",
    async () => {
      await expectOk(
        context.createBlock([
          context.polkadotApi.tx.parachainStaking
            .joinCandidates(MIN_GLMR_STAKING, 1)
            .signAsync(baltathar),
          context.polkadotApi.tx.parachainStaking
            .delegate(alith.address, MIN_GLMR_DELEGATOR, 0, 0)
            .signAsync(charleth),
          context.polkadotApi.tx.parachainStaking
            .delegate(baltathar.address, MIN_GLMR_DELEGATOR, 0, 0)
            .signAsync(ethan),
        ])
      );
    }
  );

  it("should fail", async () => {
    const block = await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegate(alith.address, MIN_GLMR_DELEGATOR, 0, 1)
        .signAsync(ethan)
    );
    expect(block.result.successful).to.be.false;
    expect(block.result.error.name).to.equal("TooLowCandidateDelegationCountToDelegate");
  });
});

describeDevMoonbeam("Staking - Delegator Join - wrong delegation hint", (context) => {
  before(
    "should setup alith+baltathar are candidates and ethan+chaleth as delegators",
    async () => {
      await expectOk(
        context.createBlock([
          context.polkadotApi.tx.parachainStaking
            .joinCandidates(MIN_GLMR_STAKING, 1)
            .signAsync(baltathar),
          context.polkadotApi.tx.parachainStaking
            .delegate(alith.address, MIN_GLMR_DELEGATOR, 0, 0)
            .signAsync(charleth),
          context.polkadotApi.tx.parachainStaking
            .delegate(baltathar.address, MIN_GLMR_DELEGATOR, 0, 0)
            .signAsync(ethan),
        ])
      );
    }
  );

  it("should fail", async () => {
    const block = await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegate(alith.address, MIN_GLMR_DELEGATOR, 1, 0)
        .signAsync(ethan)
    );
    expect(block.result.successful).to.be.false;
    expect(block.result.error.name).to.equal("TooLowDelegationCountToDelegate");
  });
});

describeDevMoonbeam("Staking - Delegator Join - valid request", (context) => {
  const numberToHex = (n: BigInt): string => `0x${n.toString(16).padStart(32, "0")}`;

  before("should delegate", async () => {
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .delegate(alith.address, MIN_GLMR_DELEGATOR, 1, 0)
          .signAsync(ethan)
      )
    );
  });

  it("should succeed", async () => {
    const delegatorState = await context.polkadotApi.query.parachainStaking.delegatorState(
      ethan.address
    );
    expect(delegatorState.unwrap().toJSON()).to.deep.equal({
      delegations: [
        {
          amount: numberToHex(MIN_GLMR_DELEGATOR),
          owner: alith.address,
        },
      ],
      id: ethan.address,
      lessTotal: 0,
      status: { active: null },
      total: numberToHex(MIN_GLMR_DELEGATOR),
    });
  });
});

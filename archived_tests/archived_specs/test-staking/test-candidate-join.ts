import "@polkadot/api-augment";
import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { MIN_GLMR_STAKING, MIN_GLMR_DELEGATOR } from "../../../util/constants";
import { describeDevMoonbeam } from "../../../util/setup-dev-tests";
import { alith, ethan } from "../../../util/accounts";
import { expectOk } from "../../../util/expect";

describeDevMoonbeam("Staking - Candidate Join - bond less than min", (context) => {
  it("should fail", async () => {
    const minCandidateStk = context.polkadotApi.consts.parachainStaking.minCandidateStk;
    const block = await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .joinCandidates(minCandidateStk.subn(10), 1)
        .signAsync(ethan)
    );
    expect(block.result.successful).to.be.false;
    expect(block.result.error.name).to.equal("CandidateBondBelowMin");
  });
});

describeDevMoonbeam("Staking - Candidate Join - already a delegator", (context) => {
  before("should delegete", async () => {
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
      context.polkadotApi.tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, 1).signAsync(ethan)
    );
    expect(block.result.successful).to.be.false;
    expect(block.result.error.name).to.equal("DelegatorExists");
  });
});

describeDevMoonbeam("Staking - Candidate Join - already a candidate", (context) => {
  it("should fail", async () => {
    const block = await context.createBlock(
      context.polkadotApi.tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, 1).signAsync(alith)
    );
    expect(block.result.successful).to.be.false;
    expect(block.result.error.name).to.equal("CandidateExists");
  });
});

describeDevMoonbeam("Staking - Candidate Join - wrong candidate delegation hint", (context) => {
  it("should fail", async () => {
    const block = await context.createBlock(
      context.polkadotApi.tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, 0).signAsync(ethan)
    );
    expect(block.result.successful).to.be.false;
    expect(block.result.error.name).to.equal("TooLowCandidateCountWeightHintJoinCandidates");
  });
});

describeDevMoonbeam("Staking - Candidate Join - valid request", (context) => {
  const numberToHex = (n: BigInt): string => `0x${n.toString(16).padStart(32, "0")}`;

  before("should join candidates", async () => {
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, 1).signAsync(ethan)
      )
    );
  });

  it("should succeed", async () => {
    const candidateState = await context.polkadotApi.query.parachainStaking.candidateInfo(
      ethan.address
    );
    expect(candidateState.unwrap().toJSON()).to.deep.equal({
      bond: numberToHex(MIN_GLMR_STAKING),
      delegationCount: 0,
      totalCounted: numberToHex(MIN_GLMR_STAKING),
      lowestTopDelegationAmount: 0,
      highestBottomDelegationAmount: 0,
      lowestBottomDelegationAmount: 0,
      topCapacity: "Empty",
      bottomCapacity: "Empty",
      request: null,
      status: { active: null },
    });
  });
});

import "@moonbeam-network/api-augment";

import { numberToHex } from "@polkadot/util";
import { expect } from "chai";

import { alith, ethan, ETHAN_PRIVATE_KEY } from "../../util/accounts";
import { verifyLatestBlockFees } from "../../util/block";
import { MIN_GLMR_STAKING, PRECOMPILE_PARACHAIN_STAKING_ADDRESS } from "../../util/constants";
import {
  describeDevMoonbeam,
  describeDevMoonbeamAllEthTxTypes,
  DevTestContext,
} from "../../util/setup-dev-tests";
import { callPrecompile, sendPrecompileTx } from "../../util/transactions";

const SELECTORS = {
  candidate_bond_less: "289b6ba7",
  candidate_bond_more: "c57bd3a8",
  go_offline: "767e0450",
  go_online: "d2f73ceb",
  is_candidate: "8545c833",
  is_selected_candidate: "8f6d27c7",
  is_delegator: "8e5080e7",
  join_candidates: "0a1bff60",
  leave_candidates: "72b02a31",
  leave_delegators: "b71d2153",
  min_nomination: "c9f593b2",
  nominate: "49df6eb3",
  nominator_bond_less: "f6a52569",
  nominator_bond_more: "971d44c8",
  revoke_nomination: "4b65c34b",
  points: "9799b4e7",
  candidate_count: "4b1c4c29",
  collator_nomination_count: "0ad6a7be",
  nominator_nomination_count: "dae5659b",
  delegation_request_is_pending: "192e1db3",
  candidate_exit_is_pending: "eb613b8a",
};

async function isSelectedCandidate(context: DevTestContext, address: string) {
  return await callPrecompile(
    context,
    PRECOMPILE_PARACHAIN_STAKING_ADDRESS,
    SELECTORS,
    "is_selected_candidate",
    [address]
  );
}

async function IsDelegator(context: DevTestContext, address: string) {
  return await callPrecompile(
    context,
    PRECOMPILE_PARACHAIN_STAKING_ADDRESS,
    SELECTORS,
    "is_delegator",
    [address]
  );
}

async function isCandidate(context: DevTestContext, address: string) {
  return await callPrecompile(
    context,
    PRECOMPILE_PARACHAIN_STAKING_ADDRESS,
    SELECTORS,
    "is_candidate",
    [address]
  );
}

async function candidateCount(context: DevTestContext) {
  return await callPrecompile(
    context,
    PRECOMPILE_PARACHAIN_STAKING_ADDRESS,
    SELECTORS,
    "candidate_count",
    []
  );
}

async function delegationRequestIsPending(
  context: DevTestContext,
  delegatorAddress: string,
  collatorAddress: string
) {
  return await callPrecompile(
    context,
    PRECOMPILE_PARACHAIN_STAKING_ADDRESS,
    SELECTORS,
    "delegation_request_is_pending",
    [delegatorAddress, collatorAddress]
  );
}

describeDevMoonbeam("Staking - Genesis", (context) => {
  it("should include collator from the specs", async function () {
    expect(Number((await isSelectedCandidate(context, alith.address)).result)).to.equal(1);
  });
  it("should have one collator", async function () {
    expect(Number((await candidateCount(context)).result)).to.equal(1);
  });
});

describeDevMoonbeamAllEthTxTypes("Staking - Join Candidates", (context) => {
  it("should successfully call joinCandidates on ethan", async function () {
    const { result } = await sendPrecompileTx(
      context,
      PRECOMPILE_PARACHAIN_STAKING_ADDRESS,
      SELECTORS,
      ethan.address,
      ETHAN_PRIVATE_KEY,
      "join_candidates",
      [numberToHex(Number(MIN_GLMR_STAKING)), numberToHex(1)]
    );

    const receipt = await context.web3.eth.getTransactionReceipt(result.hash);
    expect(receipt.status).to.equal(true);

    const candidatesAfter = await context.polkadotApi.query.parachainStaking.candidatePool();
    expect(candidatesAfter.length).to.equal(2, "New candidate should have been added");
    expect(candidatesAfter[1].owner.toString()).to.equal(
      ethan.address,
      "New candidate ethan should have been added"
    );
    expect(candidatesAfter[1].amount.toBigInt()).to.equal(
      1000000000000000000000n,
      "new candidate ethan should have been added (wrong amount)"
    );

    expect(Number((await isCandidate(context, ethan.address)).result)).to.equal(1);
    await verifyLatestBlockFees(context, 0n);
  });
});

describeDevMoonbeamAllEthTxTypes("Staking - Collator Leaving", (context) => {
  before("add ethan to candidates", async () => {
    const { result } = await sendPrecompileTx(
      context,
      PRECOMPILE_PARACHAIN_STAKING_ADDRESS,
      SELECTORS,
      ethan.address,
      ETHAN_PRIVATE_KEY,
      "join_candidates",
      [numberToHex(Number(MIN_GLMR_STAKING)), numberToHex(1)]
    );

    const receipt = await context.web3.eth.getTransactionReceipt(result.hash);
    expect(receipt.status).to.equal(true);
  });

  it("should successfully call candidate_exit_is_pending on ethan", async function () {
    const { result } = await sendPrecompileTx(
      context,
      PRECOMPILE_PARACHAIN_STAKING_ADDRESS,
      SELECTORS,
      ethan.address,
      ETHAN_PRIVATE_KEY,
      "candidate_exit_is_pending",
      [ethan.address]
    );

    const receipt = await context.web3.eth.getTransactionReceipt(result.hash);
    expect(receipt.status).to.equal(true);
  });
});

describeDevMoonbeamAllEthTxTypes("Staking - Join Delegators", (context) => {
  beforeEach("should successfully call delegate for ethan.address to ALITH", async function () {
    await sendPrecompileTx(
      context,
      PRECOMPILE_PARACHAIN_STAKING_ADDRESS,
      SELECTORS,
      ethan.address,
      ETHAN_PRIVATE_KEY,
      "nominate",
      [alith.address, numberToHex(Number(MIN_GLMR_STAKING)), "0x0", "0x0"]
    );
  });

  it("should have successfully delegated ALITH", async function () {
    const delegatorsAfter = (
      (await context.polkadotApi.query.parachainStaking.delegatorState(ethan.address)) as any
    ).unwrap();
    expect(
      (
        delegatorsAfter.toJSON() as {
          delegations: { owner: string; amount: string }[];
        }
      ).delegations[0].owner
    ).to.equal(alith.address, "delegation didn't go through");
    expect(delegatorsAfter.status.toString()).equal("Active");
  });
});

describeDevMoonbeamAllEthTxTypes("Staking - Join Delegators", (context) => {
  before("should successfully call delegate for ethan.address to ALITH", async function () {
    // Delegate ethan.address->ALITH
    await sendPrecompileTx(
      context,
      PRECOMPILE_PARACHAIN_STAKING_ADDRESS,
      SELECTORS,
      ethan.address,
      ETHAN_PRIVATE_KEY,
      "nominate",
      [alith.address, numberToHex(Number(MIN_GLMR_STAKING)), "0x0", "0x0"]
    );
  });

  it("should verify delegation pending requests", async function () {
    expect(
      Number((await delegationRequestIsPending(context, ethan.address, alith.address)).result)
    ).to.equal(0);

    // Schedule Revoke
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .scheduleRevokeDelegation(alith.address)
        .signAsync(ethan)
    );

    // Check that there exists a pending request
    expect(
      Number((await delegationRequestIsPending(context, ethan.address, alith.address)).result)
    ).to.equal(1);
  });
});

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
  GENESIS_ACCOUNT,
  ALITH_PRIV_KEY,
} from "../../util/constants";
import { blake2AsHex, randomAsHex } from "@polkadot/util-crypto";
import { describeDevMoonbeam, DevTestContext } from "../../util/setup-dev-tests";
import { numberToHex, stringToHex } from "@polkadot/util";
import Web3 from "web3";
import { customWeb3Request } from "../../util/providers";
import { callPrecompile, sendPrecompileTx } from "../../util/transactions";
import { verifyLatestBlockFees } from "../../util/block";

const ADDRESS_STAKING = "0x0000000000000000000000000000000000000800";

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
};

async function isSelectedCandidate(context: DevTestContext, address: string) {
  return await callPrecompile(context, ADDRESS_STAKING, SELECTORS, "is_selected_candidate", [
    address,
  ]);
}

async function IsDelegator(context: DevTestContext, address: string) {
  return await callPrecompile(context, ADDRESS_STAKING, SELECTORS, "is_delegator", [address]);
}

async function isCandidate(context: DevTestContext, address: string) {
  return await callPrecompile(context, ADDRESS_STAKING, SELECTORS, "is_candidate", [address]);
}

async function candidateCount(context: DevTestContext) {
  return await callPrecompile(context, ADDRESS_STAKING, SELECTORS, "candidate_count", []);
}

describeDevMoonbeam("Staking - Genesis", (context) => {
  it("should include collator from the specs", async function () {
    expect(Number((await isSelectedCandidate(context, COLLATOR_ACCOUNT)).result)).to.equal(1);
  });
  it("should have one collator", async function () {
    expect(Number((await candidateCount(context)).result)).to.equal(1);
  });
});

describeDevMoonbeam("Staking - Join Candidates", (context) => {
  it.only("should successfully call joinCandidates on ETHAN", async function () {
    const block = await sendPrecompileTx(
      context,
      ADDRESS_STAKING,
      SELECTORS,
      ETHAN,
      ETHAN_PRIVKEY,
      "join_candidates",
      [numberToHex(Number(MIN_GLMR_STAKING)), numberToHex(1)]
    );

    const receipt = await context.web3.eth.getTransactionReceipt(block.txResults[0].result);
    expect(receipt.status).to.equal(true);

    let candidatesAfter = await context.polkadotApi.query.parachainStaking.candidatePool();
    expect((candidatesAfter.toJSON() as { owner: string; amount: string }[]).length).to.equal(
      2,
      "New candidate should have been added"
    );
    expect((candidatesAfter.toJSON() as { owner: string; amount: string }[])[1].owner).to.equal(
      ETHAN,
      "New candidate ethan should have been added"
    );
    expect((candidatesAfter.toJSON() as { owner: string; amount: string }[])[1].amount).to.equal(
      "0x000000000000003635c9adc5dea00000",
      "new candidate ethan should have been added (wrong amount)"
    );

    expect(Number((await isCandidate(context, ETHAN)).result)).to.equal(1);
    await verifyLatestBlockFees(context.polkadotApi, expect, MIN_GLMR_STAKING);
  });
});

describeDevMoonbeam("Staking - Join Delegators", (context) => {
  beforeEach("should successfully call delegate for ETHAN to ALITH", async function () {
    await sendPrecompileTx(context, ADDRESS_STAKING, SELECTORS, ETHAN, ETHAN_PRIVKEY, "nominate", [
      ALITH,
      numberToHex(Number(MIN_GLMR_STAKING)),
      "0x0",
      "0x0",
    ]);
  });

  it("should have successfully delegated ALITH", async function () {
    const delegatorsAfter = (
      (await context.polkadotApi.query.parachainStaking.delegatorState(ETHAN)) as any
    ).unwrap();
    expect(
      (
        delegatorsAfter.toJSON() as {
          delegations: { owner: string; amount: string }[];
        }
      ).delegations[0].owner
    ).to.equal(ALITH, "delegation didn't go through");
    expect(delegatorsAfter.status.toString()).equal("Active");
  });
});

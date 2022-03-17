import { ethers } from "ethers";
import { expect } from "chai";

import {
  ALITH, ALITH_PRIV_KEY,
  BALTATHAR, BALTATHAR_PRIV_KEY,
  GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY
} from "../util/constants";
import { customWeb3Request } from "../util/providers";
import { getCompiled } from "../util/contracts";
import { describeDevMoonbeam, describeDevMoonbeamAllEthTxTypes } from "../util/setup-dev-tests";
import { createContract, createTransaction, callPrecompile } from "../util/transactions";

const GAS_PRICE = "0x" + (1_000_000_000).toString(16);
const ADDRESS_STAKING = "0x0000000000000000000000000000000000000800";
// TODO: DRY, this is used in test-precompile-staking
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

describeDevMoonbeam("Contract can satisfy origin", (context) => {
  it("allow contract to be a staker", async function () {
    const contractData = await getCompiled("JoinCandidatesWrapper");
    const iFace = new ethers.utils.Interface(contractData.contract.abi);
    const { contract, rawTx } = await createContract(context, "JoinCandidatesWrapper");
    const address = contract.options.address;
    await context.createBlock({ transactions: [rawTx] });

    let data = iFace.encodeFunctionData("join");

    const pay_tx = await createTransaction(context, {
      from: BALTATHAR,
      privateKey: BALTATHAR_PRIV_KEY,
      value: "0x" + (1_000_000_000_000_000_000_000).toString(16),
      gas: "0x100000",
      gasPrice: GAS_PRICE,
      to: address,
    });
    await context.createBlock({ transactions: [pay_tx] });

    const join_tx = await createTransaction(context, {
      from: BALTATHAR,
      privateKey: BALTATHAR_PRIV_KEY,
      value: "0x0",
      gas: "0x100000",
      gasPrice: GAS_PRICE,
      to: address,
      data: data,
    });
    await context.createBlock({ transactions: [join_tx] });

    let isContractCandidateResult
      = await callPrecompile(context, ADDRESS_STAKING, SELECTORS, "is_candidate", [address]);
    expect(Number(isContractCandidateResult.result)).to.equal(1);

    // baltathar sent the original transaction but it should be the contract itself that is the new
    // candidate
    let isBaltatharCandidateResult
      = await callPrecompile(context, ADDRESS_STAKING, SELECTORS, "is_candidate", [BALTATHAR]);
    expect(Number(isBaltatharCandidateResult.result)).to.equal(0);

  });
});

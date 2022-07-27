import "@moonbeam-network/api-augment";

import { expect } from "chai";
import { ethers } from "ethers";

import { alith, ethan } from "../../util/accounts";
import { verifyLatestBlockFees } from "../../util/block";
import { MIN_GLMR_STAKING, PRECOMPILE_PARACHAIN_STAKING_ADDRESS } from "../../util/constants";
import { getCompiled } from "../../util/contracts";
import { web3EthCall } from "../../util/providers";
import { describeDevMoonbeam, describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import { createTransaction, ETHAN_TRANSACTION_TEMPLATE } from "../../util/transactions";

const PARACHAIN_STAKING_CONTRACT = getCompiled("ParachainStaking");
const PARACHAIN_STAKING_INTERFACE = new ethers.utils.Interface(
  PARACHAIN_STAKING_CONTRACT.contract.abi
);

describeDevMoonbeam("Precompiles - Staking - Genesis", (context) => {
  it("should include collator from the specs", async function () {
    const { result } = await web3EthCall(context.web3, {
      to: PRECOMPILE_PARACHAIN_STAKING_ADDRESS,
      data: PARACHAIN_STAKING_INTERFACE.encodeFunctionData("is_selected_candidate", [
        alith.address,
      ]),
    });

    expect(Number(result)).to.equal(1);
  });

  it("should have one collator", async function () {
    const { result } = await web3EthCall(context.web3, {
      to: PRECOMPILE_PARACHAIN_STAKING_ADDRESS,
      data: PARACHAIN_STAKING_INTERFACE.encodeFunctionData("candidate_count"),
    });

    expect(Number(result)).to.equal(1);
  });
});

describeDevMoonbeamAllEthTxTypes("Precompiles - Staking - Join Candidates", (context) => {
  before("add ethan as candidate", async function () {
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ETHAN_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_PARACHAIN_STAKING_ADDRESS,
        data: PARACHAIN_STAKING_INTERFACE.encodeFunctionData("join_candidates", [
          MIN_GLMR_STAKING,
          1,
        ]),
      })
    );

    const receipt = await context.web3.eth.getTransactionReceipt(result.hash);
    expect(receipt.status).to.equal(true);
  });

  it("should successfully call joinCandidates on ethan", async function () {
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

    const { result } = await web3EthCall(context.web3, {
      to: PRECOMPILE_PARACHAIN_STAKING_ADDRESS,
      data: PARACHAIN_STAKING_INTERFACE.encodeFunctionData("is_candidate", [alith.address]),
    });

    expect(Number(result)).to.equal(1);
    await verifyLatestBlockFees(context, 0n);
  });
});

describeDevMoonbeamAllEthTxTypes("Precompiles - Staking - Collator Leaving", (context) => {
  before("add ethan to candidates", async () => {
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ETHAN_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_PARACHAIN_STAKING_ADDRESS,
        data: PARACHAIN_STAKING_INTERFACE.encodeFunctionData("join_candidates", [
          MIN_GLMR_STAKING,
          1,
        ]),
      })
    );

    const receipt = await context.web3.eth.getTransactionReceipt(result.hash);
    expect(receipt.status).to.equal(true);
  });

  it("should successfully call candidate_exit_is_pending on ethan", async function () {
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ETHAN_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_PARACHAIN_STAKING_ADDRESS,
        data: PARACHAIN_STAKING_INTERFACE.encodeFunctionData("candidate_exit_is_pending", [
          ethan.address,
        ]),
      })
    );

    const receipt = await context.web3.eth.getTransactionReceipt(result.hash);
    expect(receipt.status).to.equal(true);
  });
});

describeDevMoonbeamAllEthTxTypes("Precompiles - Staking - Join Delegators", (context) => {
  beforeEach("should successfully call delegate for ethan.address to ALITH", async function () {
    await context.createBlock(
      createTransaction(context, {
        ...ETHAN_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_PARACHAIN_STAKING_ADDRESS,
        data: PARACHAIN_STAKING_INTERFACE.encodeFunctionData("delegate", [
          alith.address,
          MIN_GLMR_STAKING,
          0,
          0,
        ]),
      })
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

describeDevMoonbeamAllEthTxTypes("Precompiles - Staking - Join Delegators", (context) => {
  before("should successfully call delegate for ethan.address to ALITH", async function () {
    // Delegate ethan.address->ALITH
    await context.createBlock(
      createTransaction(context, {
        ...ETHAN_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_PARACHAIN_STAKING_ADDRESS,
        data: PARACHAIN_STAKING_INTERFACE.encodeFunctionData("delegate", [
          alith.address,
          MIN_GLMR_STAKING,
          0,
          0,
        ]),
      })
    );

    const { result } = await web3EthCall(context.web3, {
      to: PRECOMPILE_PARACHAIN_STAKING_ADDRESS,
      data: PARACHAIN_STAKING_INTERFACE.encodeFunctionData("delegation_request_is_pending", [
        ethan.address,
        alith.address,
      ]),
    });

    expect(Number(result)).to.equal(0);
  });

  it("should verify delegation pending requests", async function () {
    // Schedule Revoke
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .scheduleRevokeDelegation(alith.address)
        .signAsync(ethan)
    );

    // Check that there exists a pending request
    const { result } = await web3EthCall(context.web3, {
      to: PRECOMPILE_PARACHAIN_STAKING_ADDRESS,
      data: PARACHAIN_STAKING_INTERFACE.encodeFunctionData("delegation_request_is_pending", [
        ethan.address,
        alith.address,
      ]),
    });

    expect(Number(result)).to.equal(1);
  });
});

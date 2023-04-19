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

const PARACHAIN_STAKING_CONTRACT = getCompiled("precompiles/parachain-staking/ParachainStaking");
const PARACHAIN_STAKING_INTERFACE = new ethers.utils.Interface(
  PARACHAIN_STAKING_CONTRACT.contract.abi
);

describeDevMoonbeam("Precompiles - Staking - Genesis", (context) => {
  it("should include collator from the specs", async function () {
    const { result } = await web3EthCall(context.web3, {
      to: PRECOMPILE_PARACHAIN_STAKING_ADDRESS,
      data: PARACHAIN_STAKING_INTERFACE.encodeFunctionData("isSelectedCandidate", [alith.address]),
    });

    expect(Number(result)).to.equal(1);
  });

  it("should have one collator", async function () {
    const { result } = await web3EthCall(context.web3, {
      to: PRECOMPILE_PARACHAIN_STAKING_ADDRESS,
      data: PARACHAIN_STAKING_INTERFACE.encodeFunctionData("candidateCount"),
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
        data: PARACHAIN_STAKING_INTERFACE.encodeFunctionData("joinCandidates", [
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
      data: PARACHAIN_STAKING_INTERFACE.encodeFunctionData("isCandidate", [alith.address]),
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
        data: PARACHAIN_STAKING_INTERFACE.encodeFunctionData("joinCandidates", [
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
        data: PARACHAIN_STAKING_INTERFACE.encodeFunctionData("candidateExitIsPending", [
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

  it("should have correct delegation amount for ethan to ALITH", async function () {
    // Check that delegation amount equals MIN_GLMR_STAKING
    const { result } = await web3EthCall(context.web3, {
      to: PRECOMPILE_PARACHAIN_STAKING_ADDRESS,
      data: PARACHAIN_STAKING_INTERFACE.encodeFunctionData("delegationAmount", [
        ethan.address,
        alith.address,
      ]),
    });

    expect(BigInt(result)).to.equal(MIN_GLMR_STAKING);
  });

  it("should have 0 delegation amount for delegation that DNE", async function () {
    // Check that delegation amount is 0 when delegation DNE
    const { result } = await web3EthCall(context.web3, {
      to: PRECOMPILE_PARACHAIN_STAKING_ADDRESS,
      data: PARACHAIN_STAKING_INTERFACE.encodeFunctionData("delegationAmount", [
        alith.address,
        alith.address,
      ]),
    });

    expect(BigInt(result)).to.equal(0n);
  });

  it("should have ethan's delegation to ALITH in top delegations", async function () {
    // Check that delegation is in top delegations
    const { result } = await web3EthCall(context.web3, {
      to: PRECOMPILE_PARACHAIN_STAKING_ADDRESS,
      data: PARACHAIN_STAKING_INTERFACE.encodeFunctionData("isInTopDelegations", [
        ethan.address,
        alith.address,
      ]),
    });

    expect(Number(result)).to.equal(1);
  });

  it("should not be in top delegations when delegation DNE", async function () {
    const { result } = await web3EthCall(context.web3, {
      to: PRECOMPILE_PARACHAIN_STAKING_ADDRESS,
      data: PARACHAIN_STAKING_INTERFACE.encodeFunctionData("isInTopDelegations", [
        alith.address,
        alith.address,
      ]),
    });

    expect(Number(result)).to.equal(0);
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
      data: PARACHAIN_STAKING_INTERFACE.encodeFunctionData("delegationRequestIsPending", [
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
      data: PARACHAIN_STAKING_INTERFACE.encodeFunctionData("delegationRequestIsPending", [
        ethan.address,
        alith.address,
      ]),
    });

    expect(Number(result)).to.equal(1);
  });
});

describeDevMoonbeamAllEthTxTypes("Precompiles - Staking - AwardedPoints", (context) => {
  before("should successfully produce a block by ALITH", async function () {
    await context.createBlock();
  });

  it("should get awarded points for ALITH", async function () {
    const { result } = await web3EthCall(context.web3, {
      to: PRECOMPILE_PARACHAIN_STAKING_ADDRESS,
      data: PARACHAIN_STAKING_INTERFACE.encodeFunctionData("awardedPoints", [1, alith.address]),
    });

    expect(Number(result)).to.equal(20);
  });

  it("should get no awarded points for ETHAN", async function () {
    const { result } = await web3EthCall(context.web3, {
      to: PRECOMPILE_PARACHAIN_STAKING_ADDRESS,
      data: PARACHAIN_STAKING_INTERFACE.encodeFunctionData("awardedPoints", [1, ethan.address]),
    });

    expect(Number(result)).to.equal(0);
  });
});

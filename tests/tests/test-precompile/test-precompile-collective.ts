import "@moonbeam-network/api-augment";

import { expect } from "chai";
import { ethers } from "ethers";

import {
  alith,
  baltathar,
  charleth,
  CHARLETH_ADDRESS,
  dorothy,
  DOROTHY_ADDRESS,
  ethan,
} from "../../util/accounts";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { getCompiled } from "../../util/contracts";
import {
  ALITH_TRANSACTION_TEMPLATE,
  BALTATHAR_TRANSACTION_TEMPLATE,
  CHARLETH_TRANSACTION_TEMPLATE,
  createContract,
  createTransaction,
  DOROTHY_TRANSACTION_TEMPLATE,
  TRANSACTION_TEMPLATE,
} from "../../util/transactions";
import { expectEVMResult } from "../../util/eth-transactions";
import {
  PRECOMPILE_COUNCIL_ADDRESS,
  PRECOMPILE_TREASURY_COUNCIL_ADDRESS,
} from "../../util/constants";
import { web3EthCall } from "../../util/providers";
import { blake2AsHex } from "@polkadot/util-crypto";

const COLLECTIVE_CONTRACT_JSON = getCompiled("CollectivePrecompile");
const COLLECTIVE_INTERFACE = new ethers.utils.Interface(COLLECTIVE_CONTRACT_JSON.contract.abi);

// Duplicate of treasury tests but with interaction with the collectives throught
// precompiles.

const encode = (call) => call?.method.toHex() || "";

const successfulCouncilCall = async (context, template, data) => {
  const tx = await createTransaction(context, {
    ...template,
    to: PRECOMPILE_COUNCIL_ADDRESS,
    gas: 5_000_000,
    data: data,
  });

  let { result } = await context.createBlock(tx);

  expect(result?.successful).to.equal(true);
};

const successfulTreasuryCouncilCall = async (context, template, data) => {
  const tx = await createTransaction(context, {
    ...template,
    to: PRECOMPILE_TREASURY_COUNCIL_ADDRESS,
    gas: 5_000_000,
    data: data,
  });

  let { result } = await context.createBlock(tx);

  expect(result?.successful).to.equal(true);
};

describeDevMoonbeam("Treasury council precompile #1", (context) => {
  it("should not be able to be approved by a non-council member", async function () {
    // Ethan submit a treasury proposal

    await context.createBlock(
      context.polkadotApi.tx.treasury.proposeSpend(10, baltathar.address).signAsync(ethan)
    );

    // Verify that the proposal is submitted
    let proposalCount = await context.polkadotApi.query.treasury.proposalCount();
    expect(proposalCount.toHuman() === "1").to.equal(true, "new proposal should have been added");

    // Try to approve the proposal directly (must be fail)
    let approvals = (await context.polkadotApi.query.treasury.approvals()) as any;
    expect(approvals.length).to.equal(0, "No proposal must have been approved");
  });
});

describeDevMoonbeam("Treasury council precompile #2", (context) => {
  it("should not be able to be rejected by a non-council member", async function () {
    // Ethan submit a treasury proposal

    await context.createBlock(
      context.polkadotApi.tx.treasury.proposeSpend(10, baltathar.address).signAsync(ethan)
    );

    // Verify that the proposal is submitted
    let proposalCount = await context.polkadotApi.query.treasury.proposalCount();
    expect(proposalCount.toHuman() === "1").to.equal(true, "new proposal should have been added");

    // Try to reject the proposal directly (must be fail)
    await context.polkadotApi.tx.treasury.rejectProposal(0).signAsync(ethan);
    expect(await context.polkadotApi.query.treasury.proposals(0)).not.equal(
      null,
      "The proposal should not have been deleted"
    );
  });
});

describeDevMoonbeam("Treasury council precompile #3", (context) => {
  // prettier-ignore
  it(
    "should be rejected if three-fifths of the treasury council did not vote in favor",
    async function () {
      // Ethan submit a treasury proposal

      await context.createBlock(
        context.polkadotApi.tx.treasury.proposeSpend(10, baltathar.address).signAsync(ethan)
      );

      // Verify that the proposal is submitted
      let proposalCount = await context.polkadotApi.query.treasury.proposalCount();
      expect(proposalCount.toHuman() === "1").to.equal(true, "new proposal should have been added");

      // A council member attempts to approve the proposal on behalf of the council
      const { result: evmResult } = await context.createBlock(
        createTransaction(context, {
          ...BALTATHAR_TRANSACTION_TEMPLATE,
          from: CHARLETH_ADDRESS,
          to: PRECOMPILE_TREASURY_COUNCIL_ADDRESS,
          data: COLLECTIVE_INTERFACE.encodeFunctionData("propose", [
            1,
            encode(context.polkadotApi.tx.treasury.approveProposal(0))
          ]),          
        })
      );
      expectEVMResult(evmResult.events, "Succeed");

      // Verify that the proposal is not deleted
      expect(await context.polkadotApi.query.treasury.proposals(0)).not.equal(
        null,
        "The proposal must not have been deleted"
      );
    }
  );
});

describeDevMoonbeam("Treasury council precompile #4", (context) => {
  // prettier-ignore
  it(
    "should not be rejected by less than half of the members of the treasury council",
    async function () {
      // Ethan submit a treasury proposal

      await context.createBlock(
        context.polkadotApi.tx.treasury.proposeSpend(10, baltathar.address).signAsync(ethan)
      );

      // Verify that the proposal is submitted
      let proposalCount = await context.polkadotApi.query.treasury.proposalCount();
      expect(proposalCount.toHuman() === "1").to.equal(true, "new proposal should have been added");

      // A council member attempts to reject the proposal on behalf of the council
      // (must fail because there is not a quorum)
      const { result: evmResult } = await context.createBlock(
        createTransaction(context, {
          ...BALTATHAR_TRANSACTION_TEMPLATE,
          from: CHARLETH_ADDRESS,
          to: PRECOMPILE_TREASURY_COUNCIL_ADDRESS,
          data: COLLECTIVE_INTERFACE.encodeFunctionData("propose", [
            1,
            encode(context.polkadotApi.tx.treasury.rejectProposal(0))
          ]),          
        })
      );
      expectEVMResult(evmResult.events, "Succeed");

      // Verify that the proposal is not approved
      let approvals = (await context.polkadotApi.query.treasury.approvals()) as any;
      expect(approvals.length).to.equal(0, "No proposal should have been approved");
    }
  );
});

describeDevMoonbeam("Treasury council precompile #5", (context) => {
  it("should be approvable by root", async function () {
    // Ethan submit a treasury proposal

    await context.createBlock(
      context.polkadotApi.tx.treasury.proposeSpend(10, baltathar.address).signAsync(ethan)
    );

    // Verify that the proposal is submitted
    let proposalCount = await context.polkadotApi.query.treasury.proposalCount();
    expect(proposalCount.toHuman() === "1").to.equal(true, "new proposal should have been added");

    // Root approve the proposal directly
    await context.createBlock(
      context.polkadotApi.tx.sudo
        .sudo(context.polkadotApi.tx.treasury.approveProposal(0))
        .signAsync(alith)
    );

    // Verify that the proposal is approved
    let approvals = (await context.polkadotApi.query.treasury.approvals()) as any;
    expect(approvals.length).to.equal(1, "One proposal should have been approved");
  });
});

describeDevMoonbeam("Treasury council precompile #6", (context) => {
  it("should be rejectable by root", async function () {
    // Ethan submit a treasury proposal

    await context.createBlock(
      context.polkadotApi.tx.treasury.proposeSpend(10, baltathar.address).signAsync(ethan)
    );

    // Verify that the proposal is submitted
    let proposalCount = await context.polkadotApi.query.treasury.proposalCount();
    expect(proposalCount.toHuman() === "1").to.equal(true, "new proposal should have been added");

    // Root approve the proposal directly
    await context.createBlock(
      context.polkadotApi.tx.sudo
        .sudo(context.polkadotApi.tx.treasury.rejectProposal(0))
        .signAsync(alith)
    );

    // Verify that the proposal is deleted
    expect(await (await context.polkadotApi.query.treasury.proposals(0)).toHuman()).to.equal(
      null,
      "The proposal must have been deleted"
    );
  });
});

describeDevMoonbeam("Treasury council precompile #7", (context) => {
  // prettier-ignore
  it(
    "should NO LONGER be approved if the three fifths of the council voted for it",
    async function () {
      // To run this long scenarios, we have to go through a lot of steps, so anyway we won't be
      // able to keep this tests within 5 seconds.
      this.timeout(10_000);

      // Ethan submit a treasury proposal

      await context.createBlock(
        context.polkadotApi.tx.treasury.proposeSpend(10, baltathar.address).signAsync(ethan)
      );

      // Verify that the proposal is submitted
      let proposalCount = (await context.polkadotApi.query.treasury.proposalCount()) as any;
      expect(proposalCount.toBigInt()).to.equal(1n, "new proposal should have been added");

      // Charleth submit the proposal to the council
      const proposal = encode(context.polkadotApi.tx.treasury.approveProposal(0));
      const proposalHash = blake2AsHex(proposal).toString();

      await successfulCouncilCall(
        context,
        CHARLETH_TRANSACTION_TEMPLATE,
        COLLECTIVE_INTERFACE.encodeFunctionData("propose", [
          2,
          proposal
      ]));

      // Charleth & Dorothy vote for this proposal and close it
      await successfulCouncilCall(
        context,
        CHARLETH_TRANSACTION_TEMPLATE,
        COLLECTIVE_INTERFACE.encodeFunctionData("vote", [
          proposalHash,
          0,
          true
        ])
      );

      await successfulCouncilCall(
        context,
        DOROTHY_TRANSACTION_TEMPLATE,
        COLLECTIVE_INTERFACE.encodeFunctionData("vote", [
          proposalHash,
          0,
          true
        ])
      );

      // council vote succeeds, proposal dispatch success is not
      // taken into account.
      await successfulCouncilCall(
        context,
        DOROTHY_TRANSACTION_TEMPLATE,
        COLLECTIVE_INTERFACE.encodeFunctionData("close", [
          proposalHash,
          0,
          800_000_000,
          proposal.length / 2 - 1,
        ])
      );

      // Verify that the proposal is not approved
      let approvals = (await context.polkadotApi.query.treasury.approvals()) as any;
      expect(approvals.length).to.equal(0, "No proposal should have been approved");
    }
  );
});

describeDevMoonbeam("Treasury council precompile #8", (context) => {
  // prettier-ignore
  it(
    "should NO LONGER be rejected if the half of the council voted against it",
    async function () {
      // Ethan submit a treasury proposal

      await context.createBlock(
        context.polkadotApi.tx.treasury.proposeSpend(10, baltathar.address).signAsync(ethan)
      );

      // Verify that the proposal is submitted
      let proposalCount = (await context.polkadotApi.query.treasury.proposalCount()) as any;
      expect(proposalCount.toBigInt()).to.equal(1n, "new proposal should have been added");

      // Charleth proposed that the council reject the treasury proposal
      const proposal = encode(context.polkadotApi.tx.treasury.approveProposal(0));
      const proposalHash = blake2AsHex(proposal);

      await successfulCouncilCall(
        context,
        CHARLETH_TRANSACTION_TEMPLATE,
        COLLECTIVE_INTERFACE.encodeFunctionData("propose", [
          2,
          proposal
      ]));

      // Charleth & Dorothy vote for against this proposal and close it
      await successfulCouncilCall(
        context,
        CHARLETH_TRANSACTION_TEMPLATE,
        COLLECTIVE_INTERFACE.encodeFunctionData("vote", [
          proposalHash,
          0,
          true
        ])
      );

      await successfulCouncilCall(
        context,
        DOROTHY_TRANSACTION_TEMPLATE,
        COLLECTIVE_INTERFACE.encodeFunctionData("vote", [
          proposalHash,
          0,
          true
        ])
      );

      // council vote succeeds, proposal dispatch success is not
      // taken into account.
      await successfulCouncilCall(
        context,
        DOROTHY_TRANSACTION_TEMPLATE,
        COLLECTIVE_INTERFACE.encodeFunctionData("close", [
          proposalHash,
          0,
          800_000_000,
          proposal.length / 2 - 1,
        ])
      );

      // Verify that the proposal is not deleted
      expect(await context.polkadotApi.query.treasury.proposals(0)).not.equal(
        null,
        "The proposal must not have been deleted"
      );
    }
  );
});

describeDevMoonbeam("Treasury council precompile #9", (context) => {
  // prettier-ignore
  it(
    "should be approved if the three fifths of the treasury council voted for it",
    async function () {
      // To run this long scenarios, we have to go through a lot of steps, so anyway we won't be
      // able to keep this tests within 5 seconds.
      this.timeout(10_000);

      // Ethan submit a treasury proposal

      await context.createBlock(
        context.polkadotApi.tx.treasury.proposeSpend(10, baltathar.address).signAsync(ethan)
      );

      // Verify that the proposal is submitted
      let proposalCount = (await context.polkadotApi.query.treasury.proposalCount()) as any;
      expect(proposalCount.toBigInt()).to.equal(1n, "new proposal should have been added");

      const proposal = encode(context.polkadotApi.tx.treasury.approveProposal(0));
      const proposalHash = blake2AsHex(proposal).toString();

      await successfulTreasuryCouncilCall(
        context,
        CHARLETH_TRANSACTION_TEMPLATE,
        COLLECTIVE_INTERFACE.encodeFunctionData("propose", [
          2,
          proposal
      ]));

      // Charleth & Dorothy vote for this proposal and close it
      await successfulTreasuryCouncilCall(
        context,
        CHARLETH_TRANSACTION_TEMPLATE,
        COLLECTIVE_INTERFACE.encodeFunctionData("vote", [
          proposalHash,
          0,
          true
        ])
      );

      await successfulTreasuryCouncilCall(
        context,
        DOROTHY_TRANSACTION_TEMPLATE,
        COLLECTIVE_INTERFACE.encodeFunctionData("vote", [
          proposalHash,
          0,
          true
        ])
      );

      await successfulTreasuryCouncilCall(
        context,
        DOROTHY_TRANSACTION_TEMPLATE,
        COLLECTIVE_INTERFACE.encodeFunctionData("close", [
          proposalHash,
          0,
          10_000_000_000,
          proposal.length / 2 - 1,
        ])
      );

      // Verify that the proposal is approved
      let approvals = (await context.polkadotApi.query.treasury.approvals()) as any;
      expect(approvals.length).to.equal(1, "one proposal should have been approved");
    }
  );
});

describeDevMoonbeam("Treasury council precompile #10", (context) => {
  it("should be rejected if the half of the treasury council voted against it", async function () {
    // Ethan submit a treasury proposal

    await context.createBlock(
      context.polkadotApi.tx.treasury.proposeSpend(10, baltathar.address).signAsync(ethan)
    );

    // Verify that the proposal is submitted
    let proposalCount = (await context.polkadotApi.query.treasury.proposalCount()) as any;
    expect(proposalCount.toBigInt()).to.equal(1n, "new proposal should have been added");

    // Charleth proposed that the council reject the treasury proposal
    const proposal = encode(context.polkadotApi.tx.treasury.rejectProposal(0));
    const proposalHash = blake2AsHex(proposal).toString();

    await successfulTreasuryCouncilCall(
      context,
      CHARLETH_TRANSACTION_TEMPLATE,
      COLLECTIVE_INTERFACE.encodeFunctionData("propose", [2, proposal])
    );

    // Charleth & Dorothy vote against this proposal and close it
    await successfulTreasuryCouncilCall(
      context,
      CHARLETH_TRANSACTION_TEMPLATE,
      COLLECTIVE_INTERFACE.encodeFunctionData("vote", [proposalHash, 0, true])
    );

    await successfulTreasuryCouncilCall(
      context,
      DOROTHY_TRANSACTION_TEMPLATE,
      COLLECTIVE_INTERFACE.encodeFunctionData("vote", [proposalHash, 0, true])
    );

    await successfulTreasuryCouncilCall(
      context,
      DOROTHY_TRANSACTION_TEMPLATE,
      COLLECTIVE_INTERFACE.encodeFunctionData("close", [
        proposalHash,
        0,
        10_000_000_000,
        proposal.length / 2 - 1,
      ])
    );

    // Verify that the proposal is deleted
    expect((await context.polkadotApi.query.treasury.proposals(0)).toHuman()).to.equal(
      null,
      "The proposal must have been deleted"
    );
  });
});

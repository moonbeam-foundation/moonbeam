import "@moonbeam-network/api-augment";
import {
  DevModeContext,
  beforeAll,
  beforeEach,
  describeSuite,
  expect,
  fetchCompiledContract,
} from "@moonwall/cli";
import {
  BALTATHAR_ADDRESS,
  BALTATHAR_PRIVATE_KEY,
  CHARLETH_ADDRESS,
  CHARLETH_PRIVATE_KEY,
  DOROTHY_PRIVATE_KEY,
  PRECOMPILE_COUNCIL_ADDRESS,
  PRECOMPILE_TREASURY_COUNCIL_ADDRESS,
  createViemTransaction,
  ethan,
} from "@moonwall/util";
import { blake2AsHex } from "@polkadot/util-crypto";
import { Abi, encodeFunctionData } from "viem";
import { expectEVMResult } from "../../../../helpers";

const successfulCouncilCall = async (
  context: DevModeContext,
  privateKey: `0x${string}`,
  data: `0x${string}`
) => {
  const tx = await createViemTransaction(context, {
    to: PRECOMPILE_COUNCIL_ADDRESS,
    gas: 5_000_000n,
    data: data,
    privateKey,
    skipEstimation: true,
  });

  const { result } = await context.createBlock(tx);

  expect(result?.successful).to.equal(true);
};

describeSuite({
  id: "D012829",
  title: "Treasury council precompile #1",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let collectivePrecompileAbi: Abi;
    let proposalId: bigint;

    beforeEach(async () => {
      const countBefore = (await context.polkadotJs().query.treasury.proposalCount()).toBigInt();
      await context.createBlock(
        context.polkadotJs().tx.treasury.proposeSpend(10, BALTATHAR_ADDRESS).signAsync(ethan)
      );
      const countAfter = (await context.polkadotJs().query.treasury.proposalCount()).toBigInt();
      expect(countBefore + 1n, "new proposal should have been added").toBe(countAfter);
      proposalId = countAfter - 1n;
    });

    beforeAll(async () => {
      const { abi } = fetchCompiledContract("CollectivePrecompile");
      collectivePrecompileAbi = abi;
    });

    it({
      id: "T01",
      title: "should not be able to be approved by a non-council member",
      test: async function () {
        const countBefore = (await context.polkadotJs().query.treasury.proposalCount()).toBigInt();
        // Ethan submit a treasury proposal
        await context.createBlock(
          context.polkadotJs().tx.treasury.proposeSpend(10, BALTATHAR_ADDRESS).signAsync(ethan)
        );
        const countAfter = (await context.polkadotJs().query.treasury.proposalCount()).toBigInt();
        expect(countBefore + 1n, "new proposal should have been added").toBe(countAfter);

        // Try to approve the proposal directly (must be fail)
        await context.createBlock(
          context.polkadotJs().tx.treasury.approveProposal(proposalId).signAsync(ethan),
          { allowFailures: true }
        );

        const approvals = await context.polkadotJs().query.treasury.approvals();
        expect(approvals.length).to.equal(0, "No proposal must have been approved");
      },
    });

    it({
      id: "T02",
      title: "should not be able to be rejected by a non-council member",
      test: async function () {
        // Try to reject the proposal directly (must be fail)
        await context.polkadotJs().tx.treasury.rejectProposal(proposalId).signAsync(ethan);
        expect(
          await context.polkadotJs().query.treasury.proposals(proposalId),
          "The proposal should not have been deleted"
        ).not.equal(null);
      },
    });

    it({
      id: "T03",
      title: "should be rejected if three-fifths of the treasury council did not vote in favor",
      test: async function () {
        // A council member attempts to approve the proposal on behalf of the council
        const { result: evmResult } = await context.createBlock(
          createViemTransaction(context, {
            privateKey: BALTATHAR_PRIVATE_KEY,
            from: CHARLETH_ADDRESS,
            to: PRECOMPILE_TREASURY_COUNCIL_ADDRESS,
            data: encodeFunctionData({
              abi: collectivePrecompileAbi,
              functionName: "propose",
              args: [
                1,
                context.polkadotJs().tx.treasury.approveProposal(proposalId).method.toHex(),
              ],
            }),
          })
        );
        expectEVMResult(evmResult!.events, "Succeed");

        // Verify that the proposal is not deleted
        expect(
          (await context.polkadotJs().query.treasury.proposals(proposalId)).isSome,
          "The proposal must not have been deleted"
        ).toBe(true);
      },
    });

    it({
      id: "T04",
      title: "should not be rejected by less than half of the members of the treasury council",
      test: async function () {
        const approvalsBefore = (await context.polkadotJs().query.treasury.approvals()).length;
        // A council member attempts to reject the proposal on behalf of the council
        // (must fail because there is not a quorum)
        const { result: evmResult } = await context.createBlock(
          createViemTransaction(context, {
            privateKey: BALTATHAR_PRIVATE_KEY,
            from: CHARLETH_ADDRESS,
            to: PRECOMPILE_TREASURY_COUNCIL_ADDRESS,
            data: encodeFunctionData({
              abi: collectivePrecompileAbi,
              functionName: "propose",
              args: [1, context.polkadotJs().tx.treasury.rejectProposal(proposalId).method.toHex()],
            }),
          })
        );
        expectEVMResult(evmResult!.events, "Succeed");
        const approvalsAfter = (await context.polkadotJs().query.treasury.approvals()).length;

        expect(approvalsAfter, "No proposal should have been approved").toBe(approvalsBefore);
      },
    });

    it({
      id: "T05",
      title: "should be approvable by root",
      test: async function () {
        const approvalsBefore = (await context.polkadotJs().query.treasury.approvals()).length;

        // Root approve the proposal directly
        await context.createBlock(
          context
            .polkadotJs()
            .tx.sudo.sudo(context.polkadotJs().tx.treasury.approveProposal(proposalId))
        );

        // Verify that the proposal is approved
        const approvalsAfter = (await context.polkadotJs().query.treasury.approvals()).length;
        expect(approvalsAfter - approvalsBefore, "One proposal should have been approved").to.equal(
          1
        );
      },
    });

    it({
      id: "T06",
      title: "should be rejectable by root",
      test: async function () {
        await context.createBlock(
          context
            .polkadotJs()
            .tx.sudo.sudo(context.polkadotJs().tx.treasury.rejectProposal(proposalId))
        );

        expect(
          (await context.polkadotJs().query.treasury.proposals(proposalId)).isNone,
          "The proposal must has not been deleted"
        ).toBe(true);
      },
    });

    it({
      id: "T07",
      title: "should NO LONGER be approved if the three fifths of the council voted for it",
      timeout: 10_000,
      test: async function () {
        const approvalsCountBefore = (await context.polkadotJs().query.treasury.approvals()).length;

        // Charleth submit the proposal to the council
        const proposal = context
          .polkadotJs()
          .tx.treasury.approveProposal(proposalId)
          .method.toHex();
        const proposalHash = blake2AsHex(proposal).toString();

        await successfulCouncilCall(
          context,
          CHARLETH_PRIVATE_KEY,
          encodeFunctionData({
            abi: collectivePrecompileAbi,
            functionName: "propose",
            args: [2, proposal],
          })
        );

        // Charleth & Dorothy vote for this proposal and close it
        await successfulCouncilCall(
          context,
          CHARLETH_PRIVATE_KEY,
          encodeFunctionData({
            abi: collectivePrecompileAbi,
            functionName: "vote",
            args: [proposalHash, proposalId, true],
          })
        );

        await successfulCouncilCall(
          context,
          DOROTHY_PRIVATE_KEY,
          encodeFunctionData({
            abi: collectivePrecompileAbi,
            functionName: "vote",
            args: [proposalHash, proposalId, true],
          })
        );

        // council vote succeeds, proposal dispatch success is not
        // taken into account.
        await successfulCouncilCall(
          context,
          DOROTHY_PRIVATE_KEY,
          encodeFunctionData({
            abi: collectivePrecompileAbi,
            functionName: "close",
            args: [proposalHash, proposalId, 800_000_000, proposal.length / 2 - 1],
          })
        );

        // Verify that the proposal is not approved
        const approvalsCountAfter = (await context.polkadotJs().query.treasury.approvals()).length;
        expect(approvalsCountAfter, "No proposal should have been approved").toBe(
          approvalsCountBefore
        );
      },
    });

    it({
      id: "T08",
      title: "should NO LONGER be rejected if the half of the council voted against it",
      test: async function () {
        // Charleth proposed that the council reject the treasury proposal
        const proposal = context.polkadotJs().tx.treasury.approveProposal(0).method.toHex();
        const proposalHash = blake2AsHex(proposal);

        await successfulCouncilCall(
          context,
          CHARLETH_PRIVATE_KEY,
          encodeFunctionData({
            abi: collectivePrecompileAbi,
            functionName: "propose",
            args: [2, proposal],
          })
        );

        // Charleth & Dorothy vote for against this proposal and close it
        await successfulCouncilCall(
          context,
          CHARLETH_PRIVATE_KEY,
          encodeFunctionData({
            abi: collectivePrecompileAbi,
            functionName: "vote",
            args: [proposalHash, proposalId, true],
          })
        );

        await successfulCouncilCall(
          context,
          DOROTHY_PRIVATE_KEY,
          encodeFunctionData({
            abi: collectivePrecompileAbi,
            functionName: "vote",
            args: [proposalHash, proposalId, true],
          })
        );

        // council vote succeeds, proposal dispatch success is not
        // taken into account.
        await successfulCouncilCall(
          context,
          DOROTHY_PRIVATE_KEY,
          encodeFunctionData({
            abi: collectivePrecompileAbi,
            functionName: "close",
            args: [proposalHash, proposalId, 800_000_000, proposal.length / 2 - 1],
          })
        );

        // Verify that the proposal is not deleted
        expect(
          (await context.polkadotJs().query.treasury.proposals(proposalId)).isSome,
          "The proposal shouldn't be deleted"
        ).toBe(true);
      },
    });
  },
});

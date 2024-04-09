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
  CHARLETH_PRIVATE_KEY,
  DOROTHY_PRIVATE_KEY,
  PRECOMPILE_TREASURY_COUNCIL_ADDRESS,
  createViemTransaction,
  ethan,
} from "@moonwall/util";
import { blake2AsHex } from "@polkadot/util-crypto";
import { Abi, encodeFunctionData } from "viem";

const successfulTreasuryCouncilCall = async (
  context: DevModeContext,
  privateKey: `0x${string}`,
  data: `0x${string}`
) => {
  const tx = await createViemTransaction(context, {
    to: PRECOMPILE_TREASURY_COUNCIL_ADDRESS,
    gas: 5_000_000n,
    data: data,
    privateKey,
    skipEstimation: true,
  });

  const { result } = await context.createBlock(tx);

  expect(result?.successful).to.equal(true);
};
describeSuite({
  id: "D012830",
  title: "Treasury council precompile #2",
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
      title: "should be approved if the three fifths of the treasury council voted for it",
      timeout: 10_000,
      test: async function () {
        const approvalsCountBefore = (await context.polkadotJs().query.treasury.approvals()).length;
        const proposal = context
          .polkadotJs()
          .tx.treasury.approveProposal(proposalId)
          .method.toHex();
        const proposalHash = blake2AsHex(proposal).toString();

        await successfulTreasuryCouncilCall(
          context,
          CHARLETH_PRIVATE_KEY,
          encodeFunctionData({
            abi: collectivePrecompileAbi,
            functionName: "propose",
            args: [2, proposal],
          })
        );

        // Charleth & Dorothy vote for this proposal and close it
        await successfulTreasuryCouncilCall(
          context,
          CHARLETH_PRIVATE_KEY,
          encodeFunctionData({
            abi: collectivePrecompileAbi,
            functionName: "vote",
            args: [proposalHash, proposalId, true],
          })
        );

        await successfulTreasuryCouncilCall(
          context,
          DOROTHY_PRIVATE_KEY,
          encodeFunctionData({
            abi: collectivePrecompileAbi,
            functionName: "vote",
            args: [proposalHash, proposalId, true],
          })
        );

        await successfulTreasuryCouncilCall(
          context,
          DOROTHY_PRIVATE_KEY,
          encodeFunctionData({
            abi: collectivePrecompileAbi,
            functionName: "close",
            args: [proposalHash, proposalId, 10_000_000_000, proposal.length / 2 - 1],
          })
        );
        // Verify that the proposal is approved
        const approvalsCountAfter = (await context.polkadotJs().query.treasury.approvals()).length;
        expect(approvalsCountBefore + 1, "one proposal should have been approved").toBe(
          approvalsCountAfter
        );
      },
    });

    it({
      id: "T02",
      title: "should be rejected if the half of the treasury council voted against it",
      test: async function () {
        // Charleth proposed that the council reject the treasury proposal
        const proposal = context.polkadotJs().tx.treasury.rejectProposal(proposalId).method.toHex();
        const proposalHash = blake2AsHex(proposal).toString();

        await successfulTreasuryCouncilCall(
          context,
          CHARLETH_PRIVATE_KEY,
          encodeFunctionData({
            abi: collectivePrecompileAbi,
            functionName: "propose",
            args: [2, proposal],
          })
        );

        // Charleth & Dorothy vote against this proposal and close it
        await successfulTreasuryCouncilCall(
          context,
          CHARLETH_PRIVATE_KEY,
          encodeFunctionData({
            abi: collectivePrecompileAbi,
            functionName: "vote",
            args: [proposalHash, proposalId, true],
          })
        );

        await successfulTreasuryCouncilCall(
          context,
          DOROTHY_PRIVATE_KEY,
          encodeFunctionData({
            abi: collectivePrecompileAbi,
            functionName: "vote",
            args: [proposalHash, proposalId, true],
          })
        );

        await successfulTreasuryCouncilCall(
          context,
          DOROTHY_PRIVATE_KEY,
          encodeFunctionData({
            abi: collectivePrecompileAbi,
            functionName: "close",
            args: [proposalHash, proposalId, 10_000_000_000, proposal.length / 2 - 1],
          })
        );

        // Verify that the proposal is deleted
        expect(
          (await context.polkadotJs().query.treasury.proposals(proposalId)).isNone,
          "The proposal should be deleted"
        ).toBe(true);
      },
    });
  },
});

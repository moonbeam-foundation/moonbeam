import "@moonbeam-network/api-augment";
import {
  BALTATHAR_ADDRESS,
  BALTATHAR_PRIVATE_KEY,
  CHARLETH_ADDRESS,
  CHARLETH_PRIVATE_KEY,
  PRECOMPILE_TREASURY_COUNCIL_ADDRESS,
  beforeAll,
  createViemTransaction,
  describeSuite,
  expect,
  fetchCompiledContract,
} from "moonwall";
import { type Abi, encodeFunctionData } from "viem";
import { expectEVMResult, expectSubstrateEvent } from "../../../../helpers";

describeSuite({
  id: "D010415",
  title: "Treasury council precompile #1",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let collectivePrecompileAbi: Abi;

    beforeAll(async () => {
      const { abi } = fetchCompiledContract("CollectivePrecompile");
      collectivePrecompileAbi = abi;
    });

    it({
      id: "T01",
      title: "The treasury council proposal works",
      test: async function () {
        const proposal = context
          .polkadotJs()
          .tx.treasury.spend(null as any, 10, BALTATHAR_ADDRESS, null as any)
          .method.toHex();

        // A council member attempts to approve the proposal on behalf of the council
        const block = await context.createBlock(
          createViemTransaction(context, {
            privateKey: BALTATHAR_PRIVATE_KEY,
            to: PRECOMPILE_TREASURY_COUNCIL_ADDRESS,
            data: encodeFunctionData({
              abi: collectivePrecompileAbi,
              functionName: "propose",
              args: [2, proposal],
            }),
          })
        );
        expectEVMResult(block.result!.events, "Succeed");
        const proposedEvent = expectSubstrateEvent(block as any, "treasuryCouncilCollective", "Proposed");
        const proposalResult: any = proposedEvent.data.toHuman();
        expect(proposalResult).toMatchObject({
          account: BALTATHAR_ADDRESS,
          proposalIndex: "0",
          proposalHash: proposalResult.proposalHash,
          threshold: "2",
        });

        // Baltathar and Charleth vote for this proposal
        const block2 = await context.createBlock(
          createViemTransaction(context, {
            privateKey: BALTATHAR_PRIVATE_KEY,
            to: PRECOMPILE_TREASURY_COUNCIL_ADDRESS,
            data: encodeFunctionData({
              abi: collectivePrecompileAbi,
              functionName: "vote",
              args: [proposalResult.proposalHash, proposalResult.proposalIndex, true],
            }),
          })
        );
        expectEVMResult(block2.result!.events, "Succeed");
        let votedEvent = expectSubstrateEvent(block2 as any, "treasuryCouncilCollective", "Voted");
        expect(votedEvent.data.toHuman()).toMatchObject({
          account: BALTATHAR_ADDRESS,
          proposalHash: proposalResult.proposalHash,
          voted: true,
          yes: "1",
          no: "0",
        });

        const block3 = await context.createBlock(
          createViemTransaction(context, {
            privateKey: CHARLETH_PRIVATE_KEY,
            to: PRECOMPILE_TREASURY_COUNCIL_ADDRESS,
            data: encodeFunctionData({
              abi: collectivePrecompileAbi,
              functionName: "vote",
              args: [proposalResult.proposalHash, proposalResult.proposalIndex, true],
            }),
          })
        );
        expectEVMResult(block3.result!.events, "Succeed");
        votedEvent = expectSubstrateEvent(block3 as any, "treasuryCouncilCollective", "Voted");
        expect(votedEvent.data.toHuman()).toMatchObject({
          account: CHARLETH_ADDRESS,
          proposalHash: proposalResult.proposalHash,
          voted: true,
          yes: "2",
          no: "0",
        });

        // Charleth closes the proposal
        const block4 = await context.createBlock(
          createViemTransaction(context, {
            privateKey: CHARLETH_PRIVATE_KEY,
            to: PRECOMPILE_TREASURY_COUNCIL_ADDRESS,
            gas: 8_000_000n,
            data: encodeFunctionData({
              abi: collectivePrecompileAbi,
              functionName: "close",
              args: [
                proposalResult.proposalHash,
                proposalResult.proposalIndex,
                800_000_000,
                proposal.slice(2).length / 2,
              ],
            }),
          })
        );

        expectEVMResult(block4.result!.events, "Succeed");
        const approvedEvent = expectSubstrateEvent(block4 as any, "treasuryCouncilCollective", "Approved");
        const closedEvent = expectSubstrateEvent(block4 as any, "treasuryCouncilCollective", "Closed");
        const assetSpendApprovedEvent = expectSubstrateEvent(
          block4 as any,
          "treasury",
          "AssetSpendApproved"
        );
        expect(approvedEvent.data.toHuman()).toMatchObject({
          proposalHash: proposalResult.proposalHash,
        });
        expect(closedEvent.data.toHuman()).toMatchObject({
          proposalHash: proposalResult.proposalHash,
          yes: "2",
          no: "0",
        });
        expect(assetSpendApprovedEvent.data.toHuman()).toMatchObject({
          amount: "10",
          assetKind: "Native",
          beneficiary: "0x3Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0",
          expireAt: "432,004",
          index: "0",
          validFrom: "4",
        });
      },
    });
  },
});

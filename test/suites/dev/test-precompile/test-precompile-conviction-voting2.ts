import "@moonbeam-network/api-augment";
import { beforeEach, describeSuite, expect } from "@moonwall/cli";
import { expectEVMResult } from "../../../helpers/eth-transactions.js";
import { createProposal } from "../../../helpers/voting.js";

const CONVICTION_VALUES = [0n, 1n, 2n, 3n, 4n, 5n, 6n];

describeSuite({
  id: "D2529-1",
  title: "Precompiles - Conviction",
  foundationMethods: "dev",
  testCases: ({ it, log, context }) => {
    let proposalIndex: number;
    beforeEach(async function () {
      proposalIndex = await createProposal(context);
    });

    for (const conviction of CONVICTION_VALUES) {
      it({
        id: "T01",
        title: `should allow to vote with confiction x${conviction}`,
        test: async function () {
          const rawTxn = await context.writePrecompile!({
            precompileName: "ConvictionVoting",
            functionName: "voteYes",
            args: [proposalIndex, 1n * 10n ** 18n, conviction],
            rawTxOnly: true,
          });

          const block = await context.createBlock(rawTxn);
          expectEVMResult(block.result!.events, "Succeed");

          // Verifies the substrate side
          const referendum = await context
            .polkadotJs()
            .query.referenda.referendumInfoFor(proposalIndex);
          expect(referendum.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(
            1n * 10n ** 17n * (conviction == 0n ? 1n : conviction * 10n)
          );
        },
      });
    }
  },
});

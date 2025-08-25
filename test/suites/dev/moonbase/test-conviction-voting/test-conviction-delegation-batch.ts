import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect, proposeReferendaAndDeposit } from "@moonwall/cli";
import { ALITH_ADDRESS, GLMR, alith, baltathar } from "@moonwall/util";

describeSuite({
  id: "D020603",
  title: "Conviction Voting - Batch Delegation",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    const proposals: [number, string][] = [];
    const origins: [number, any][] = [
      [0, { System: "root" }],
      [1, { Origins: "WhitelistedCaller" }],
      [2, { Origins: "GeneralAdmin" }],
      [3, { Origins: "ReferendumCanceller" }],
      [4, { Origins: "ReferendumKiller" }],
    ];
    beforeAll(async () => {
      // make a proposal for each origin
      for (const [id, origin] of origins) {
        const proposal = context
          .polkadotJs()
          .tx.identity.setIdentity({ display: { raw: `Props ${id}` } });
        proposals.push(await proposeReferendaAndDeposit(context, alith, proposal, origin));
      }
    });

    it({
      id: "T01",
      title: "Alith should be able to delegate to Baltathar in batch",
      test: async function () {
        const blockResult = await context.createBlock(
          // in the same block delegate to all proposals
          context
            .polkadotJs()
            .tx.utility.batchAll(
              origins.map(([id, _], i) =>
                context
                  .polkadotJs()
                  .tx.convictionVoting.delegate(id, baltathar.address, "Locked1x", 100n * GLMR)
              )
            ),
          { allowFailures: false }
        );

        // the tests does not check for the events, but they should be there
        // expectSubstrateEvent(blockResult, "conviction  Voting", "Delegated");

        // check that all proposals have been delegated to baltathar
        await Promise.all(
          origins.map(async ([id, _], i) => {
            const votingFor = await context
              .polkadotJs()
              .query.convictionVoting.votingFor(ALITH_ADDRESS, id);

            expect(votingFor.isDelegating).toBe(true);
            expect(votingFor.asDelegating.target.toString()).toBe(baltathar.address);
            expect(votingFor.asDelegating.balance.toBigInt()).toBe(100n * GLMR);
          })
        );
      },
    });
  },
});

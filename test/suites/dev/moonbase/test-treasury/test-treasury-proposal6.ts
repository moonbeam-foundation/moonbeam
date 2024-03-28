import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { alith, baltathar, ethan } from "@moonwall/util";

describeSuite({
  id: "D013807",
  title: "Treasury proposal #6",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should be rejectable by root",
      test: async function () {
        await context.createBlock(
          context.polkadotJs().tx.treasury.proposeSpend(10, baltathar.address).signAsync(ethan)
        );

        const proposalCount = await context.polkadotJs().query.treasury.proposalCount();
        expect(proposalCount.toBigInt(), "new proposal should have been added").toBe(1n);

        // Root reject the proposal directly
        await context.createBlock(
          context
            .polkadotJs()
            .tx.sudo.sudo(context.polkadotJs().tx.treasury.rejectProposal(0))
            .signAsync(alith),
          { expectEvents: [context.polkadotJs().events.treasury.Rejected] }
        );

        expect(
          (await context.polkadotJs().query.treasury.proposals(0)).isNone,
          "The proposal hasn't been removed"
        ).toBe(true);
      },
    });
  },
});

import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect, proposeReferendaAndDeposit } from "@moonwall/cli";
import { ALITH_ADDRESS, GLMR, alith, baltathar, faith } from "@moonwall/util";
import { expectSubstrateEvent } from "../../../../helpers";

describeSuite({
  id: "D020604",
  title: "Conviction Voting - Delegation",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let refIndex: number;
    let proposalHash: string;
    beforeAll(async () => {
      // The proposal itself
      const proposal = context.polkadotJs().tx.identity.setIdentity({ display: { raw: "Me" } });
      [refIndex, proposalHash] = await proposeReferendaAndDeposit(context, alith, proposal, {
        System: "root",
      });
    });

    it({
      id: "T01",
      title: "Alith should be able to delegate to Baltathar",
      test: async function () {
        const rootTrack = context
          .polkadotJs()
          .consts.referenda.tracks.find(([, track]) => track.name.eq("root"))!;

        const blockResult = await context.createBlock(
          context
            .polkadotJs()
            .tx.convictionVoting.delegate(rootTrack[0], baltathar.address, "Locked1x", 100n * GLMR)
        );

        expectSubstrateEvent(blockResult, "convictionVoting", "Delegated");
        const votingFor = await context
          .polkadotJs()
          .query.convictionVoting.votingFor(ALITH_ADDRESS, rootTrack[0]);
        expect(votingFor.isDelegating).toBe(true);
        expect(votingFor.asDelegating.target.toString()).toBe(baltathar.address);
        expect(votingFor.asDelegating.balance.toBigInt()).toBe(100n * GLMR);
      },
    });

    it({
      id: "T02",
      title: "Alith should not be able to delegate to the same track twice",
      test: async function () {
        const rootTrack = context
          .polkadotJs()
          .consts.referenda.tracks.find(([, track]) => track.name.eq("root"))!;

        const blockResult = await context.createBlock(
          context
            .polkadotJs()
            .tx.convictionVoting.delegate(rootTrack[0], faith.address, "Locked1x", 100n * GLMR)
        );

        expectSubstrateEvent(blockResult, "system", "ExtrinsicFailed");
      },
    });

    it({
      id: "T03",
      title: "Alith should be able to undelegate",
      test: async function () {
        const rootTrack = context
          .polkadotJs()
          .consts.referenda.tracks.find(([, track]) => track.name.eq("root"))!;

        const blockResult = await context.createBlock(
          await context.polkadotJs().tx.convictionVoting.undelegate(rootTrack[0])
        );

        expectSubstrateEvent(blockResult, "convictionVoting", "Undelegated");
      },
    });
  },
});

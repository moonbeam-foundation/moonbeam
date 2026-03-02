import "@moonbeam-network/api-augment";
import { MIN_GLMR_STAKING, alith, beforeAll, describeSuite, ethan, expect } from "moonwall";
import { jumpRounds } from "../../../../helpers";

describeSuite({
  id: "D023310",
  title: "Staking - Candidate Leave Execute - after round delay",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    beforeAll(async () => {
      await context.createBlock(
        [
          context
            .polkadotJs()
            .tx.sudo.sudo(context.polkadotJs().tx.parachainStaking.setBlocksPerRound(10))
            .signAsync(alith),
          context
            .polkadotJs()
            .tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, 1)
            .signAsync(ethan),
        ],
        { signer: alith, allowFailures: false }
      );

      await context.createBlock(
        context.polkadotJs().tx.parachainStaking.scheduleLeaveCandidates(2).signAsync(ethan),
        { signer: alith, allowFailures: false }
      );

      const leaveDelay = context.polkadotJs().consts.parachainStaking.leaveDelegatorsDelay;
      await jumpRounds(context, leaveDelay.addn(5).toNumber());
    });

    it({
      id: "T01",
      title: "should succeed",
      test: async () => {
        const block = await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.executeLeaveCandidates(ethan.address, 0)
            .signAsync(ethan)
        );
        expect(block.result!.successful).to.be.true;
        const leaveEvents = block.result!.events.reduce(
          (acc, event) => {
            if (context.polkadotJs().events.parachainStaking.CandidateLeft.is(event.event)) {
              acc.push({
                account: event.event.data[0].toString(),
              });
            }
            return acc;
          },
          [] as { account: string }[]
        );

        expect(leaveEvents).to.deep.equal([
          {
            account: ethan.address,
          },
        ]);
      },
    });
  },
});

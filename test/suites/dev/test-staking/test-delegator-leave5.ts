import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { MIN_GLMR_DELEGATOR, MIN_GLMR_STAKING, alith, baltathar, ethan } from "@moonwall/util";
import { jumpRounds } from "../../../helpers/block.js";

describeSuite({
  id: "D2947",
  title: "Staking - Delegator Leave - executeLeaveDelegators executed after round delay",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const EXTRA_BOND_AMOUNT = 1_000_000_000_000_000_000n;
    const BOND_AMOUNT = MIN_GLMR_STAKING + EXTRA_BOND_AMOUNT;

    beforeAll(async () => {
      await context.createBlock(
        [
          context
            .polkadotJs()
            .tx.sudo.sudo(context.polkadotJs().tx.parachainStaking.setBlocksPerRound(10))
            .signAsync(alith),
          context
            .polkadotJs()
            .tx.parachainStaking.delegate(alith.address, MIN_GLMR_DELEGATOR, 0, 0)
            .signAsync(ethan),
        ],
        { allowFailures: false }
      );

      await context.createBlock(
        context.polkadotJs().tx.parachainStaking.scheduleLeaveDelegators().signAsync(ethan),
        { allowFailures: false }
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
            .tx.parachainStaking.executeLeaveDelegators(ethan.address, 1)
            .signAsync(ethan)
        );
        expect(block.result!.successful).to.be.true;
        const leaveEvents = block.result!.events.reduce((acc, event) => {
          if (context.polkadotJs().events.parachainStaking.DelegatorLeft.is(event.event)) {
            acc.push({
              account: event.event.data[0].toString(),
            });
          }
          return acc;
        }, []);

        expect(leaveEvents).to.deep.equal([
          {
            account: ethan.address,
          },
        ]);
      },
    });
  },
});

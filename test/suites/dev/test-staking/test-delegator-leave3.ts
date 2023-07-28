import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { MIN_GLMR_DELEGATOR, MIN_GLMR_STAKING, alith, baltathar, ethan } from "@moonwall/util";
import { jumpRounds } from "../../../helpers/block.js";


describeSuite({
  id: "D2945",
  title: "Staking - Delegator Leave Execute - before round delay",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
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
      await jumpRounds(context, leaveDelay.subn(1).toNumber());
    });

    it({
      id: "T01",
      title: "should fail",
      test: async () => {
        const block = await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.executeLeaveDelegators(ethan.address, 1)
            .signAsync(ethan)
        );
        expect(block.result!.successful).to.be.false;
        expect(block.result!.error!.name).to.equal("DelegatorCannotLeaveYet");
      },
    });
  },
});

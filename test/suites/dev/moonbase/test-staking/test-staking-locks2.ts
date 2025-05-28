import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { MIN_GLMR_STAKING, baltathar } from "@moonwall/util";
import { getCollatorStakingFreeze } from "../../../../helpers";

describeSuite({
  id: "D013476",
  title: "Staking - Freezes - join candidates",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: 'should set collator staking freeze when joining candidates',
      test: async function () {
        await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, 1)
            .signAsync(baltathar)
        );

        const stakingFreeze = await getCollatorStakingFreeze(baltathar.address as `0x${string}`, context);
        expect(stakingFreeze).to.be.equal(
          MIN_GLMR_STAKING,
          `Should have freeze for collator staking`
        );
      },
    });
  },
});

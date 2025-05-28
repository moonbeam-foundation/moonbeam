import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { GLMR, MIN_GLMR_DELEGATOR, alith, generateKeyringPair } from "@moonwall/util";
import { getDelegatorStakingFreeze } from "../../../../helpers";

describeSuite({
  id: "D013479",
  title: "Staking - Freezes - schedule revoke",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const randomAccount = generateKeyringPair();

    beforeAll(async function () {
      await context.createBlock(
        context
          .polkadotJs()
          .tx.balances.transferAllowDeath(randomAccount.address, MIN_GLMR_DELEGATOR + GLMR),
        { allowFailures: false }
      );

      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.delegateWithAutoCompound(
            alith.address,
            MIN_GLMR_DELEGATOR,
            0,
            1,
            0,
            0
          )
          .signAsync(randomAccount),
        { allowFailures: false }
      );
    });

    it({
      id: "T01",
      title: "should stay frozen after requesting a delegation revoke",
      test: async function () {
        await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.scheduleRevokeDelegation(alith.address)
            .signAsync(randomAccount),
          { allowFailures: false }
        );

        const stakingFreeze = await getDelegatorStakingFreeze(randomAccount.address as `0x${string}`, context);
        expect(stakingFreeze).to.be.equal(MIN_GLMR_DELEGATOR, "Funds should still be frozen after scheduling revoke");
      },
    });
  },
});

import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { GLMR, MIN_GLMR_DELEGATOR, alith, generateKeyringPair } from "@moonwall/util";
import {
  jumpRounds,
  getDelegatorStakingFreeze,
  getNumberOfDelegatorFreezes,
  verifyDelegatorStateMatchesFreezes,
} from "../../../../helpers";

describeSuite({
  id: "D023482",
  title: "Staking - Freezes - execute revoke",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const randomAccount = generateKeyringPair();

    beforeAll(async function () {
      await context.createBlock(
        [
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(randomAccount.address, MIN_GLMR_DELEGATOR + 1n * GLMR),
        ],
        { allowFailures: false }
      );

      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.delegateWithAutoCompound(
            alith.address,
            MIN_GLMR_DELEGATOR,
            0,
            10,
            0,
            10
          )
          .signAsync(randomAccount),
        { allowFailures: false }
      );
    });

    it({
      id: "T01",
      title: "should be thawed only after executing revoke delegation",
      timeout: 60_000,
      test: async function () {
        const freeze = await getDelegatorStakingFreeze(
          randomAccount.address as `0x${string}`,
          context
        );
        expect(freeze).to.be.equal(MIN_GLMR_DELEGATOR, "Freeze should have been added");
        
        // Verify initial state matches freezes
        await verifyDelegatorStateMatchesFreezes(randomAccount.address as `0x${string}`, context);

        await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.scheduleRevokeDelegation(alith.address)
            .signAsync(randomAccount),
          { allowFailures: false }
        );

        await jumpRounds(
          context,
          context.polkadotJs().consts.parachainStaking.revokeDelegationDelay.toNumber()
        );

        await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.executeDelegationRequest(randomAccount.address, alith.address)
            .signAsync(randomAccount),
          { allowFailures: false }
        );

        const freeze_count = await getNumberOfDelegatorFreezes(
          randomAccount.address as `0x${string}`,
          context
        );
        expect(freeze_count).to.be.equal(
          0,
          "Freeze should have been removed after executing revoke"
        );
        
        // Verify that after revoke, no delegator state exists and no freeze exists
        await verifyDelegatorStateMatchesFreezes(randomAccount.address as `0x${string}`, context);
      },
    });
  },
});

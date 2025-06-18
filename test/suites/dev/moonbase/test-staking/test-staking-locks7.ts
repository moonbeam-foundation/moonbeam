import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { GLMR, MIN_GLMR_DELEGATOR, alith, generateKeyringPair } from "@moonwall/util";
import { jumpRounds } from "../../../../helpers";

describeSuite({
  id: "D023482",
  title: "Staking - Locks - execute revoke",
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
      title: "should be unlocked only after executing revoke delegation",
      timeout: 60_000,
      test: async function () {
        const lock = await context.polkadotJs().query.balances.locks(randomAccount.address);
        expect(lock.length).to.be.equal(1, "Lock should have been added");

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

        const newLocks = await context.polkadotJs().query.balances.locks(randomAccount.address);
        expect(newLocks.length).to.be.equal(
          0,
          "Lock should have been removed after executing revoke"
        );
      },
    });
  },
});

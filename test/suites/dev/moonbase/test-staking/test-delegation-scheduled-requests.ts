import "@moonbeam-network/api-augment";
import { describeSuite, beforeAll, expect } from "@moonwall/cli";
import { MIN_GLMR_DELEGATOR, alith, ethan } from "@moonwall/util";

const numberToHex = (n: bigint): string => `0x${n.toString(16).padStart(32, "0")}`;

describeSuite({
  id: "D013319",
  title: "Staking - Delegation Scheduled Requests - schedule revoke",
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
        { signer: alith, allowFailures: false }
      );
    });

    it({
      id: "T01",
      title: "should succeed",
      test: async () => {
        // Create an extra block to prevent mismatches while comparing 'whenExecutable' field.
        await context.createBlock();

        const currentRound = (
          await context.polkadotJs().query.parachainStaking.round()
        ).current.toNumber();

        await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.scheduleRevokeDelegation(alith.address)
            .signAsync(ethan)
        );

        const delegationRequestsAfter = await context
          .polkadotJs()
          .query.parachainStaking.delegationScheduledRequests(alith.address);

        const roundDelay = context
          .polkadotJs()
          .consts.parachainStaking.revokeDelegationDelay.toNumber();

        expect(delegationRequestsAfter.toJSON()).to.deep.equal([
          {
            delegator: ethan.address,
            whenExecutable: currentRound + roundDelay,
            action: {
              revoke: numberToHex(MIN_GLMR_DELEGATOR),
            },
          },
        ]);
      },
    });
  },
});

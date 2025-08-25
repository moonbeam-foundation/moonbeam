import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { MIN_GLMR_DELEGATOR, alith, ethan } from "@moonwall/util";

describeSuite({
  id: "D023448",
  title: "Staking - Delegator Join - valid request",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const numberToHex = (n: bigint): string => `0x${n.toString(16).padStart(32, "0")}`;

    beforeAll(async () => {
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
          .signAsync(ethan),
        { allowFailures: false }
      );
    });

    it({
      id: "T01",
      title: "should succeed",
      test: async () => {
        const delegatorState = await context
          .polkadotJs()
          .query.parachainStaking.delegatorState(ethan.address);
        expect(delegatorState.unwrap().delegations[0].amount.toBigInt()).toBe(MIN_GLMR_DELEGATOR);
        expect(delegatorState.unwrap().delegations[0].owner.toString()).toBe(alith.address);
        expect(delegatorState.unwrap().id.toString()).toBe(ethan.address);
        expect(delegatorState.unwrap().lessTotal.toNumber()).toBe(0);
        expect(delegatorState.unwrap().status.isActive).toBe(true);
        expect(delegatorState.unwrap().total.toBigInt()).toBe(MIN_GLMR_DELEGATOR);
      },
    });
  },
});

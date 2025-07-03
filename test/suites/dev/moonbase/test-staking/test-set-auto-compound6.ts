import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { MIN_GLMR_DELEGATOR, alith, ethan } from "@moonwall/util";

describeSuite({
  id: "D023472",
  title: "Staking - Set Auto-Compound - remove existing config if 0% auto-compound",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.delegateWithAutoCompound(
            alith.address,
            MIN_GLMR_DELEGATOR,
            10,
            0,
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
        const autoCompoundConfigBefore = await context
          .polkadotJs()
          .query.parachainStaking.autoCompoundingDelegations(alith.address);
        expect(autoCompoundConfigBefore.isEmpty).toBe(false);
        expect(autoCompoundConfigBefore[0].value.toBigInt()).to.equal(10n);

        const { result } = await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.setAutoCompound(alith.address, 0, 1, 1)
            .signAsync(ethan)
        );
        expect(result!.successful).to.be.true;

        const autoCompoundConfigAfter = await context
          .polkadotJs()
          .query.parachainStaking.autoCompoundingDelegations(alith.address);

        expect(autoCompoundConfigAfter.isEmpty).toBe(true);
        const delegationAutoCompoundEvents = result!.events.reduce((acc, event) => {
          if (context.polkadotJs().events.parachainStaking.AutoCompoundSet.is(event.event)) {
            acc.push({
              candidate: event.event.data.candidate.toString(),
              delegator: event.event.data.delegator.toString(),
              value: event.event.data.value.toBigInt(),
            });
          }
          return acc;
        }, []);

        expect(delegationAutoCompoundEvents).to.deep.equal([
          {
            candidate: alith.address,
            delegator: ethan.address,
            value: 0n,
          },
        ]);
      },
    });
  },
});

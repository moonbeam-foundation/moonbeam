import "@moonbeam-network/api-augment";
import { MIN_GLMR_STAKING, alith, beforeAll, describeSuite, ethan, expect } from "moonwall";

describeSuite({
  id: "D023343",
  title: "Staking - Bond More - revoke scheduled",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.delegateWithAutoCompound(
            alith.address,
            MIN_GLMR_STAKING * 5n,
            0,
            0,
            0,
            0
          )
          .signAsync(ethan),
        { allowFailures: false }
      );

      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.scheduleRevokeDelegation(alith.address)
          .signAsync(ethan),
        { allowFailures: false }
      );
    });

    it({
      id: "T01",
      title: "should fail",
      test: async () => {
        const bondAmountBefore = (
          await context.polkadotJs().query.parachainStaking.delegatorState(ethan.address)
        ).unwrap().total;

        const increaseAmount = 5n;
        const block = await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.delegatorBondMore(alith.address, increaseAmount)
            .signAsync(ethan)
        );

        expect(block.result!.error!.name).to.equal("PendingDelegationRevoke");
        const bondAmountAfter = (
          await context.polkadotJs().query.parachainStaking.delegatorState(ethan.address)
        ).unwrap().total;
        expect(bondAmountAfter.eq(bondAmountBefore)).to.be.true;
      },
    });
  },
});

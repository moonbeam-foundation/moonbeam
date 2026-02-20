import "@moonbeam-network/api-augment";
import { MIN_GLMR_DELEGATOR, alith, beforeAll, describeSuite, ethan, expect } from "moonwall";

describeSuite({
  id: "D023345",
  title: "Staking - Delegator Join - already delegated",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    beforeAll(async () => {
      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.delegateWithAutoCompound(
            alith.address,
            MIN_GLMR_DELEGATOR,
            0,
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
      title: "should fail",
      test: async () => {
        const block = await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.delegateWithAutoCompound(
              alith.address,
              MIN_GLMR_DELEGATOR,
              0,
              0,
              0,
              1
            )
            .signAsync(ethan)
        );
        expect(block.result!.successful!).to.be.false;
        expect(block.result!.error!.name).to.equal("AlreadyDelegatedCandidate");
      },
    });
  },
});

import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { MIN_GLMR_DELEGATOR, alith, ethan } from "@moonwall/util";

describeSuite({
  id: "D013467",
  title: "Staking - Set Auto-Compound - new config 101%",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    beforeAll(async () => {
      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.delegateWithAutoCompound(alith.address, MIN_GLMR_DELEGATOR, 0, 0, 0, 0)
          .signAsync(ethan),
        { allowFailures: false }
      );
    });

    it({
      id: "T01",
      title: "should fail",
      test: async () => {
        expect(
          async () =>
            await context.createBlock(
              await context
                .polkadotJs()
                .tx.parachainStaking.setAutoCompound(alith.address, 101, 0, 1)
                .signAsync(ethan)
            )
        ).rejects.toThrowError("Value is greater than allowed maximum!");
      },
    });
  },
});

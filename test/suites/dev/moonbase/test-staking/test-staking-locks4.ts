import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { GLMR, MIN_GLMR_STAKING, alith, generateKeyringPair } from "@moonwall/util";

describeSuite({
  id: "D023480",
  title: "Staking - Freezes - candidate balance is frozen",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const randomAccount = generateKeyringPair();

    beforeAll(async function () {
      await context.createBlock(
        context
          .polkadotJs()
          .tx.balances.transferAllowDeath(randomAccount.address, MIN_GLMR_STAKING + GLMR),
        { allowFailures: false }
      );

      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, 1)
          .signAsync(randomAccount),
        { allowFailures: false }
      );
    });

    it({
      id: "T01",
      title: "should not be reusable for transfer",
      test: async function () {
        const { result } = await context.createBlock(
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(alith.address, MIN_GLMR_STAKING)
            .signAsync(randomAccount)
        );
        expect(result!.error!.name.toString()).to.be.equal('{"token":"Frozen"}');
      },
    });
  },
});

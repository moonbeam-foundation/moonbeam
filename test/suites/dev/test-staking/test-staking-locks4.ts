import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { GLMR, MIN_GLMR_STAKING, alith, generateKeyringPair } from "@moonwall/util";

describeSuite({
  id: "D2978",
  title: "Staking - Locks - candidate balance is locked",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const randomAccount = generateKeyringPair();

    beforeAll(async function () {
      await context.createBlock(
        context.polkadotJs().tx.balances.transfer(randomAccount.address, MIN_GLMR_STAKING + GLMR),
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
            .tx.balances.transfer(alith.address, MIN_GLMR_STAKING)
            .signAsync(randomAccount)
        );
        expect(result!.error!.name.toString()).to.be.equal('{"token":"Frozen"}');
      },
    });
  },
});

import "@moonbeam-network/api-augment";
import {
  GLMR,
  MIN_GLMR_DELEGATOR,
  alith,
  baltathar,
  beforeAll,
  describeSuite,
  expect,
  generateKeyringPair,
} from "moonwall";

describeSuite({
  id: "D023379",
  title: "Staking - Locks - delegator balance is locked",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const randomAccount = generateKeyringPair();

    beforeAll(async function () {
      await context.createBlock(
        context
          .polkadotJs()
          .tx.balances.transferAllowDeath(randomAccount.address, MIN_GLMR_DELEGATOR + GLMR),
        { allowFailures: false }
      );

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
          .signAsync(randomAccount),
        { allowFailures: false }
      );
    });

    it({
      id: "T01",
      title: "should not be reusable for delegation",
      test: async function () {
        const { result } = await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.delegateWithAutoCompound(
              baltathar.address,
              MIN_GLMR_DELEGATOR,
              0,
              10,
              0,
              10
            )
            .signAsync(randomAccount)
        );
        expect(result!.error!.name.toString()).to.be.equal("InsufficientBalance");
      },
    });
  },
});

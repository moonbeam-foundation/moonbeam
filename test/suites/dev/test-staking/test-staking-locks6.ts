import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { GLMR, MIN_GLMR_DELEGATOR, alith, generateKeyringPair } from "@moonwall/util";
import { fromBytes } from "viem";

describeSuite({
  id: "D2980",
  title: "Staking - Locks - schedule revoke",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const randomAccount = generateKeyringPair();

    beforeAll(async function () {
      await context.createBlock(
        context.polkadotJs().tx.balances.transfer(randomAccount.address, MIN_GLMR_DELEGATOR + GLMR),
        { allowFailures: false }
      );

      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.delegate(alith.address, MIN_GLMR_DELEGATOR, 1, 0)
          .signAsync(randomAccount),
        { allowFailures: false }
      );
    });

    it({
      id: "T01",
      title: "should stay locked after requesting a delegation revoke",
      test: async function () {
        await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.scheduleRevokeDelegation(alith.address)
            .signAsync(randomAccount),
          { allowFailures: false }
        );

        // Additional check
        const locks = await context.polkadotJs().query.balances.locks(randomAccount.address);
        expect(fromBytes(locks[0].id.toU8a(), "string")).to.be.equal("stkngdel");
      },
    });
  },
});

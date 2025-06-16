import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { GLMR, MIN_GLMR_DELEGATOR, alith, generateKeyringPair } from "@moonwall/util";
import { getDelegatorStakingFreeze } from "../../../../helpers";

describeSuite({
  id: "D013472",
  title: "Staking - Freezes - join delegators",
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
    });

    it({
      id: "T01",
      title: "should set freeze when delegating",
      test: async function () {
        const { result } = await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.delegateWithAutoCompound(
              alith.address,
              MIN_GLMR_DELEGATOR,
              0,
              10,
              0,
              10
            )
            .signAsync(randomAccount)
        );
        expect(result!.successful).to.be.true;

        const delegatorFreeze = await getDelegatorStakingFreeze(
          randomAccount.address as `0x${string}`,
          context
        );
        expect(delegatorFreeze > 0n, "Missing freeze").to.be.true;
        expect(delegatorFreeze).to.be.equal(MIN_GLMR_DELEGATOR);
      },
    });
  },
});

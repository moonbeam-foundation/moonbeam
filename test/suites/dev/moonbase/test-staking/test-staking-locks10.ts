import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import {
  GLMR,
  MIN_GLMR_DELEGATOR,
  MIN_GLMR_STAKING,
  alith,
  baltathar,
  generateKeyringPair,
} from "@moonwall/util";
import { getDelegatorStakingFreeze, getNumberOfDelegatorFreezes } from "../../../../helpers/staking-freezes";

describeSuite({
  id: "D013473",
  title: "Staking - Freezes - multiple delegations single freeze",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const randomAccount = generateKeyringPair();

    beforeAll(async function () {
      await context.createBlock(
        [
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(
              randomAccount.address,
              MIN_GLMR_STAKING * 2n + 1n * GLMR
            ),
          context
            .polkadotJs()
            .tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, 1)
            .signAsync(baltathar),
        ],
        { allowFailures: false }
      );

      let nonce = await context
        .viem()
        .getTransactionCount({ address: randomAccount.address as `0x${string}` });
      await context.createBlock(
        [
          context
            .polkadotJs()
            .tx.parachainStaking.delegateWithAutoCompound(
              alith.address,
              MIN_GLMR_DELEGATOR,
              100,
              10,
              10,
              10
            )
            .signAsync(randomAccount, { nonce: nonce++ }),
          context
            .polkadotJs()
            .tx.parachainStaking.delegateWithAutoCompound(
              baltathar.address,
              MIN_GLMR_DELEGATOR,
              100,
              10,
              10,
              10
            )
            .signAsync(randomAccount, { nonce: nonce++ }),
        ],
        { allowFailures: false }
      );
    });

    it({
      id: "T01",
      title: "should have a single freeze for multiple delegations",
      test: async function () {
        // Check that there's only a single delegator freeze (not multiple)
        const delegatorFreezeCount = await getNumberOfDelegatorFreezes(randomAccount.address as `0x${string}`, context);
        expect(delegatorFreezeCount).to.be.equal(
          1,
          `Should have only 1 delegator freeze, got ${delegatorFreezeCount}`
        );
      },
    });

    it({
      id: "T02",
      title: "should increase for additional delegations",
      test: async function () {
        // The freeze amount should be the sum of all delegations
        const stakingFreeze = await getDelegatorStakingFreeze(randomAccount.address as `0x${string}`, context);
        expect(stakingFreeze, `Unexpected amount for freeze`).to.be.equal(
          2n * MIN_GLMR_DELEGATOR
        );
      },
    });
  },
});

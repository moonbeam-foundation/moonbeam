import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import {
  MIN_GLMR_DELEGATOR,
  MIN_GLMR_STAKING,
  alith,
  baltathar,
  generateKeyringPair,
} from "@moonwall/util";
import { fromBytes } from "viem";
import { jumpRounds } from "../../../../helpers";

describeSuite({
  id: "D013481",
  title: "Staking - Locks - multiple delegations single revoke",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const randomAccount = generateKeyringPair();

    beforeAll(async function () {
      await context.createBlock(
        [
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(randomAccount.address, 2n * MIN_GLMR_STAKING),
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
              0,
              10,
              0,
              10
            )
            .signAsync(randomAccount, { nonce: nonce++ }),
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
            .signAsync(randomAccount, { nonce: nonce++ }),
        ],
        { allowFailures: false }
      );

      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.scheduleRevokeDelegation(alith.address)
          .signAsync(randomAccount),
        { allowFailures: false }
      );

      const locks = await context.polkadotJs().query.balances.locks(randomAccount.address);
      expect(locks.length).to.be.equal(1, "missing lock");
      expect(locks[0].amount.toBigInt()).to.equal(2n * MIN_GLMR_DELEGATOR);

      await jumpRounds(
        context,
        context.polkadotJs().consts.parachainStaking.revokeDelegationDelay.toNumber()
      );

      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.executeDelegationRequest(randomAccount.address, alith.address)
          .signAsync(randomAccount),
        { allowFailures: false }
      );
    }, 120000);

    it({
      id: "T01",
      title: "should be removed only after executing the last revoke delegation",
      test: async function () {
        // Additional check we still have 1 delegation
        const delegatorState = await context
          .polkadotJs()
          .query.parachainStaking.delegatorState(randomAccount.address);
        expect(delegatorState.unwrap().delegations.length).to.be.equal(1, "Missing delegation");
        // Only 1 over the 2 delegations has been revoked
        const locks = await context.polkadotJs().query.balances.locks(randomAccount.address);
        expect(locks.length).to.be.equal(1, "Missing lock");
        expect(locks[0].amount.toBigInt()).to.be.equal(MIN_GLMR_DELEGATOR);
        expect(fromBytes(locks[0].id.toU8a(), "string")).to.be.equal("stkngdel");
      },
    });
  },
});

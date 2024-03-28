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
import { fromBytes } from "viem";

describeSuite({
  id: "D013473",
  title: "Staking - Locks - multiple delegations single lock",
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
      title: "should not be created for additional delegations",
      test: async function () {
        const locks = await context.polkadotJs().query.balances.locks(randomAccount.address);
        expect(locks.length).to.be.equal(
          1,
          `Unexpected number of locks: ${locks.map((l) => l.id.toString()).join(` - `)}`
        );
      },
    });

    it({
      id: "T02",
      title: "should increase for additional delegations",
      test: async function () {
        const locks = await context.polkadotJs().query.balances.locks(randomAccount.address);
        expect(fromBytes(locks[0].id.toU8a(), "string")).to.be.equal("stkngdel");
        expect(locks[0].amount.toBigInt(), `Unexpected amount for lock`).to.be.equal(
          2n * MIN_GLMR_DELEGATOR
        );
      },
    });
  },
});

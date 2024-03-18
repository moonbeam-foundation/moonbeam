import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { GLMR, MIN_GLMR_DELEGATOR, alith, generateKeyringPair } from "@moonwall/util";
import { fromBytes } from "viem";

describeSuite({
  id: "D013372",
  title: "Staking - Locks - join delegators",
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
      title: 'should set "stkngdel" when delegating',
      test: async function () {
        const { result } = await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.delegate(alith.address, MIN_GLMR_DELEGATOR, 10, 10)
            .signAsync(randomAccount)
        );
        const locks = await context.polkadotJs().query.balances.locks(randomAccount.address);
        expect(result!.successful).to.be.true;
        expect(locks.length).to.be.equal(1, "Missing lock");
        expect(locks[0].amount.toBigInt()).to.be.equal(MIN_GLMR_DELEGATOR);
        expect(fromBytes(locks[0].id.toU8a(), "string")).to.be.equal("stkngdel");
      },
    });
  },
});

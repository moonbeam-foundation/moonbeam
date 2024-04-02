import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { MIN_GLMR_STAKING, baltathar } from "@moonwall/util";
import { fromBytes } from "viem";

describeSuite({
  id: "D013476",
  title: "Staking - Locks - join candidates",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: 'should set "stkngcol" when when joining candidates',
      test: async function () {
        await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, 1)
            .signAsync(baltathar)
        );

        const locks = await context.polkadotJs().query.balances.locks(baltathar.address);
        expect(locks.length).to.be.equal(
          1,
          `Unexpected number of locks: ${locks.map((l) => l.id.toHuman()).join(` - `)}`
        );
        expect(locks[0].amount.toBigInt()).to.be.equal(MIN_GLMR_STAKING);
        expect(fromBytes(locks[0].id.toU8a(), "string")).to.be.equal("stkngcol");
      },
    });
  },
});

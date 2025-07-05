import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import {
  BALTATHAR_ADDRESS,
  DEFAULT_GENESIS_BALANCE,
  MIN_GLMR_DELEGATOR,
  alith,
  baltathar,
  checkBalance,
} from "@moonwall/util";
import fs from "node:fs";
import { jumpRounds, getRewardedAndCompoundedEvents } from "../../../../helpers";

describeSuite({
  id: "D023419",
  title: "Test auto-compound with reserved balance",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    it({
      id: "T01",
      title: "should be able to auto-compound with reserved funds only (without free balance)",
      test: async function () {
        expect(
          await checkBalance(context, BALTATHAR_ADDRESS),
          "Balance should be untouched from genesis amount"
        ).toBe(DEFAULT_GENESIS_BALANCE);

        // Submit a huge preimage (to have a deposit greater than the staked amount)
        const wasm = fs.readFileSync(
          "../target/release/wbuild/moonbase-runtime/moonbase_runtime.compact.compressed.wasm"
        );
        const encodedPreimage = `0x${wasm.toString("hex")}`;
        await context.createBlock(
          context.polkadotJs().tx.preimage.notePreimage(encodedPreimage).signAsync(baltathar)
        );

        // Stake some tokens (less than the preimage deposit) with auto-compound
        const { result } = await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.delegateWithAutoCompound(
              alith.address,
              MIN_GLMR_DELEGATOR,
              50,
              0,
              0,
              0
            )
            .signAsync(baltathar)
        );
        expect(result!.successful).to.be.true;

        // Get current free balance after delegation
        const accountInfo = await context.polkadotJs().query.system.account(baltathar.address);
        const freeBalance = (accountInfo as any).data.free.toBigInt();

        // Get the existential deposit from the runtime
        const existentialDeposit = context
          .polkadotJs()
          .consts.balances.existentialDeposit.toBigInt();

        // The new freeze system requires maintaining ED in free balance when setting freezes
        const transferAmount = freeBalance - existentialDeposit;

        // Transfer all free balance except the existential deposit
        await context.createBlock(
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(alith.address, transferAmount)
            .signAsync(baltathar)
        );

        // Move forward to rewardDelay rounds
        const rewardDelay = context.polkadotJs().consts.parachainStaking.rewardPaymentDelay;
        await jumpRounds(context, rewardDelay.addn(1).toNumber());

        // The next block should reward baltathar and auto-compound his rewards
        const blockHash = (await context.createBlock()).block.hash.toString();
        const events = await getRewardedAndCompoundedEvents(context, blockHash);
        const rewardedEvent = events.rewarded.find(
          ({ account }: any) => account === baltathar.address
        );
        const compoundedEvent = events.compounded.find(
          ({ delegator }: any) => delegator === baltathar.address
        );

        expect(rewardedEvent, "delegator was not rewarded").to.not.be.undefined;
        expect(compoundedEvent, "delegator reward was not auto-compounded").to.not.be.undefined;
      },
    });
  },
});

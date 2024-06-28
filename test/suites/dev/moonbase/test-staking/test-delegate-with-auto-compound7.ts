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
  id: "D013490",
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

        // Submit a huge preimage (to have a deposit greather than the satked amount)
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

        // Withdraw all Baltathar free founds
        await context.createBlock(
          context.polkadotJs().tx.balances.transferAll(alith.address, false).signAsync(baltathar)
        );

        // Move forward to rewardDelay rounds
        const rewardDelay = context.polkadotJs().consts.parachainStaking.rewardPaymentDelay;
        await jumpRounds(context, rewardDelay.addn(1).toNumber());

        // The enxt block should reward baltathat and auto-compound his rewards
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

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
import { blake2AsHex } from "@polkadot/util-crypto";
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

        // Submit a preimage (to have a reserve that exceed min stake)
        const wasm = fs.readFileSync("../target/release/wbuild/moonbeam-runtime/moonbeam_runtime.compact.compressed.wasm");
        const encodedPreimage = `0x${wasm.toString("hex")}`;
        const encodedHash = blake2AsHex(encodedPreimage);
        await context.createBlock(
          context.polkadotJs().tx.preimage.notePreimage(encodedPreimage).signAsync(baltathar)
        );
        const reservedBalance = (await context.polkadotJs().query.system.account(BALTATHAR_ADDRESS))
            .data.reserved.toBigInt();

        // Stake some tokens (less than the preimage deposit)
        const freeBalance = (await context.polkadotJs().query.system.account(BALTATHAR_ADDRESS))
          .data.free.toBigInt();
        console.log(freeBalance);

        // Auto compound
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
        const frozenBalance = (await context.polkadotJs().query.system.account(BALTATHAR_ADDRESS))
        .data.frozen.toBigInt();

        // Withdraw all Baltathar free founds
        await context.createBlock(
          context
            .polkadotJs()
            .tx.balances.transferAll(alith.address, false)
            .signAsync(baltathar)
        );

        // Move forward to rewardDelay rounds
        const rewardDelay = context.polkadotJs().consts.parachainStaking.rewardPaymentDelay;
        await jumpRounds(context, rewardDelay.addn(1).toNumber());

        // The enxt block should reward baltathat and auto-compound his rewards
        const blockHash = (await context.createBlock()).block.hash.toString();
        const events = await getRewardedAndCompoundedEvents(context, blockHash);
        const rewardedEvent = events.rewarded.find(({ account }: any) => account === baltathar.address);
        const compoundedEvent = events.compounded.find(
          ({ delegator }: any) => delegator === baltathar.address
        );

        expect(rewardedEvent, "delegator was not rewarded").to.not.be.undefined;
        expect(compoundedEvent, "delegator reward was not auto-compounded").to.not.be.undefined;
      },
    });
  },
});

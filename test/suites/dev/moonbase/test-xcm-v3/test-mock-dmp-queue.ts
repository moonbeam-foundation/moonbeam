import "@moonbeam-network/api-augment";
import { customDevRpcRequest, describeSuite, expect } from "@moonwall/cli";
import { GLMR } from "@moonwall/util";
import type { XcmVersionedXcm } from "@polkadot/types/lookup";
import { u8aToHex } from "@polkadot/util";
import { XcmFragment, weightMessage } from "../../../../helpers";

describeSuite({
  id: "D014008",
  title: "Mock XCMP - test XCMP execution",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "Should test DMP on_initialization and on_idle",
      test: async function () {
        const metadata = await context.polkadotJs().rpc.state.getMetadata();
        const balancesPalletIndex = metadata.asLatest.pallets
          .find(({ name }) => name.toString() == "Balances")!
          .index.toNumber();

        // TODO this test mostly changes it's nature due to proof size accounting
        // by now we just decrease the number of supported messages from 50 to 20.
        const numMsgs = 20;
        // let's target half of then being executed

        // xcmp reserved is BLOCK/4
        const totalDmpWeight =
          context.polkadotJs().consts.system.blockWeights.maxBlock.refTime.toBigInt() / 4n;

        // we want half of numParaMsgs to be executed. That give us how much each message weights
        const weightPerMessage = (totalDmpWeight * BigInt(2)) / BigInt(numMsgs);

        // Now we need to construct the message. This needs to:
        // - pass barrier (withdraw + buyExecution + unlimited buyExecution*n)
        // we know at least 2 instructions are needed per message (withdrawAsset + buyExecution)
        // how many unlimited buyExecutions do we need to append?

        // we will bias this number. The reason is we want to test the decay, and therefore we need
        // an unbalanced number of messages executed. We specifically need that at some point
        // we get out of the loop of the execution (we reach the threshold limit), to then
        // go on idle

        const config = {
          assets: [
            {
              multilocation: {
                parents: 0,
                interior: {
                  X1: { PalletInstance: balancesPalletIndex },
                },
              },
              fungible: 1_000_000_000_000_000n,
            },
          ],
        };

        // How much does the withdraw weight?
        const withdrawWeight = await weightMessage(
          context,
          context
            .polkadotJs()
            .createType("XcmVersionedXcm", new XcmFragment(config).withdraw_asset().as_v3())
        );

        // How much does the buyExecution weight?
        const buyExecutionWeight = await weightMessage(
          context,
          context
            .polkadotJs()
            .createType("XcmVersionedXcm", new XcmFragment(config).buy_execution().as_v3())
        );

        // How much does the refundSurplus weight?
        // We use refund surplus because it has 0 pov
        // it's easier to focus on reftime
        const refundSurplusWeight = await weightMessage(
          context,
          context
            .polkadotJs()
            .createType("XcmVersionedXcm", new XcmFragment(config).refund_surplus().as_v3())
        );

        const xcmMessage = new XcmFragment(config).withdraw_asset().buy_execution().as_v3();

        const receivedMessage: XcmVersionedXcm = context
          .polkadotJs()
          .createType("XcmVersionedXcm", xcmMessage) as any;

        const totalMessage = [...receivedMessage.toU8a()];

        // We want these isntructions to fail in BuyExecution. That means
        // WithdrawAsset needs to work. The only way for this to work
        // is to fund each sovereign account
        const sovereignAddress = u8aToHex(
          new Uint8Array([...new TextEncoder().encode("Parent")])
        ).padEnd(42, "0");

        // We first fund the parent sovereign account with 1000
        // we will only withdraw 1, so no problem on this
        await context.createBlock(
          context.polkadotJs().tx.balances.transferAllowDeath(sovereignAddress, 1n * GLMR),
          { allowFailures: false }
        );

        // now we start injecting messages
        // several
        for (let i = 0; i < numMsgs; i++) {
          await customDevRpcRequest("xcm_injectDownwardMessage", [totalMessage]);
        }

        await context.createBlock();

        const signedBlock = await context.polkadotJs().rpc.chain.getBlock();
        const apiAt = await context.polkadotJs().at(signedBlock.block.header.hash);
        const allRecords = await apiAt.query.system.events();

        // lets grab at which point the dmp queue was exhausted
        const exhaustIndex = allRecords.findIndex(({ event }) =>
          context.polkadotJs().events.dmpQueue.MaxMessagesExhausted.is(event)
        );

        expect(
          exhaustIndex,
          "Index not found where dmpQueue is exhausted"
        ).to.be.greaterThanOrEqual(0);

        // OnInitialization
        const eventsExecutedOnInitialization = allRecords.filter(
          ({ event }, index) =>
            context.polkadotJs().events.dmpQueue.ExecutedDownward.is(event) && index < exhaustIndex
        );

        // OnIdle
        const eventsExecutedOnIdle = allRecords.filter(
          ({ event }, index) =>
            context.polkadotJs().events.dmpQueue.ExecutedDownward.is(event) && index > exhaustIndex
        );

        // the test was designed to go half and half
        expect(eventsExecutedOnInitialization.length).to.be.eq(10);
        expect(eventsExecutedOnIdle.length).to.be.eq(10);
        const pageIndex = await apiAt.query.dmpQueue.pageIndex();
        expect(pageIndex.beginUsed.toBigInt()).to.eq(0n);
        expect(pageIndex.endUsed.toBigInt()).to.eq(0n);

        // Repeat the test with different parameters
        {
          const xcmMessage = new XcmFragment(config)
            .withdraw_asset()
            .buy_execution(0, 5n)
            .refund_surplus()
            .as_v3();

          const receivedMessage: XcmVersionedXcm = context
            .polkadotJs()
            .createType("XcmVersionedXcm", xcmMessage) as any;

          const totalMessage = [...receivedMessage.toU8a()];

          // We want these isntructions to fail in BuyExecution. That means
          // WithdrawAsset needs to work. The only way for this to work
          // is to fund each sovereign account
          const sovereignAddress = u8aToHex(
            new Uint8Array([...new TextEncoder().encode("Parent")])
          ).padEnd(42, "0");

          // We first fund the parent sovereign account with 1000
          // we will only withdraw 1, so no problem on this
          await context.createBlock(
            context.polkadotJs().tx.balances.transferAllowDeath(sovereignAddress, 1n * GLMR),
            { allowFailures: false }
          );

          // now we start injecting messages
          // several
          for (let i = 0; i < numMsgs; i++) {
            await customDevRpcRequest("xcm_injectDownwardMessage", [totalMessage]);
          }

          await context.createBlock();

          const signedBlock = await context.polkadotJs().rpc.chain.getBlock();
          const apiAt = await context.polkadotJs().at(signedBlock.block.header.hash);
          console.log("signedBlock", signedBlock.block.header.hash.toHex());
          const allRecords = await apiAt.query.system.events();

          // lets grab at which point the dmp queue was exhausted
          const exhaustIndex = allRecords.findIndex(({ event }) =>
            context.polkadotJs().events.dmpQueue.MaxMessagesExhausted.is(event)
          );

          expect(
            exhaustIndex,
            "Index not found where dmpQueue is exhausted"
          ).to.be.greaterThanOrEqual(0);

          // OnInitialization
          const eventsExecutedOnInitialization = allRecords.filter(
            ({ event }, index) =>
              context.polkadotJs().events.dmpQueue.ExecutedDownward.is(event) &&
              index < exhaustIndex
          );

          // OnIdle
          const eventsExecutedOnIdle = allRecords.filter(
            ({ event }, index) =>
              context.polkadotJs().events.dmpQueue.ExecutedDownward.is(event) &&
              index > exhaustIndex
          );

          // the test was designed to go half and half
          expect(eventsExecutedOnInitialization.length).to.be.eq(10);
          expect(eventsExecutedOnIdle.length).to.be.eq(10);
          const pageIndex = await apiAt.query.dmpQueue.pageIndex();
          expect(pageIndex.beginUsed.toBigInt()).to.eq(0n);
          expect(pageIndex.endUsed.toBigInt()).to.eq(0n);
        }
      },
    });
  },
});

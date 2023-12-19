import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { CHARLETH_ADDRESS } from "../../util/accounts";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import {
  XcmFragment,
  injectHrmpMessage,
  RawXcmMessage,
  sovereignAccountOfSibling,
} from "../../util/xcm";
import { expectEVMResult } from "../../util/eth-transactions";
import { ALITH_TRANSACTION_TEMPLATE, createTransaction } from "../../util/transactions";

describeDevMoonbeam(
  "XCM V3 - Max Weight Instructions",
  (context) => {
    let dotAsset: any;
    let amount: bigint;
    const paraId: number = 888;

    before("Set up initial constants", async function () {
      const paraSovereign = sovereignAccountOfSibling(context, paraId);
      const metadata = await context.polkadotApi.rpc.state.getMetadata();
      const balancesPalletIndex = metadata.asLatest.pallets
        .find(({ name }) => name.toString() === "Balances")!
        .index.toNumber();

      // Send some native tokens to the sovereign account of paraId (to pay fees)
      const { result } = await context.createBlock(
        createTransaction(context, {
          ...ALITH_TRANSACTION_TEMPLATE,
          value: 1_000_000_000_000_000_000,
          to: paraSovereign,
          data: "0x",
        })
      );
      expectEVMResult(result.events, "Succeed");

      amount = 1_000_000_000_000_000n;
      dotAsset = {
        assets: [
          {
            multilocation: {
              parents: 0,
              interior: {
                X1: { PalletInstance: balancesPalletIndex },
              },
            },
            fungible: amount,
          },
        ],
        // weight_limit: new BN(4000000000),
        beneficiary: CHARLETH_ADDRESS,
      };
    });

    // WEIGHT::MAX instructions
    it("Should not execute UniversalOrigin", async function () {
      const xcmMessage = new XcmFragment(dotAsset)
        .withdraw_asset()
        .buy_execution()
        .universal_origin({ Parachain: paraId })
        .as_v3();

      // Mock the reception of the xcm message
      await injectHrmpMessage(context, paraId, {
        type: "XcmVersionedXcm",
        payload: xcmMessage,
      } as RawXcmMessage);
      await context.createBlock();

      // Search for WeightNotComputable error
      const events = (await context.polkadotApi.query.system.events()).filter(({ event }) =>
        context.polkadotApi.events.xcmpQueue.Fail.is(event)
      );
      expect(events).to.have.lengthOf(1);
      expect(events[0].toHuman().event["data"]["error"]).equals("WeightNotComputable");
    });

    it("Should not execute ExportMessage", async function () {
      const xcmMessage = new XcmFragment(dotAsset)
        .withdraw_asset()
        .buy_execution()
        .export_message()
        .as_v3();

      // Mock the reception of the xcm message
      await injectHrmpMessage(context, paraId, {
        type: "XcmVersionedXcm",
        payload: xcmMessage,
      } as RawXcmMessage);
      await context.createBlock();

      // Search for WeightNotComputable error
      const events = (await context.polkadotApi.query.system.events()).filter(({ event }) =>
        context.polkadotApi.events.xcmpQueue.Fail.is(event)
      );
      expect(events).to.have.lengthOf(1);
      expect(events[0].toHuman().event["data"]["error"]).equals("WeightNotComputable");
    });

    it("Should not execute LockAsset", async function () {
      const xcmMessage = new XcmFragment(dotAsset)
        .withdraw_asset()
        .buy_execution()
        .lock_asset()
        .as_v3();

      // Mock the reception of the xcm message
      await injectHrmpMessage(context, paraId, {
        type: "XcmVersionedXcm",
        payload: xcmMessage,
      } as RawXcmMessage);
      await context.createBlock();

      // Search for WeightNotComputable error
      const events = (await context.polkadotApi.query.system.events()).filter(({ event }) =>
        context.polkadotApi.events.xcmpQueue.Fail.is(event)
      );
      expect(events).to.have.lengthOf(1);
      expect(events[0].toHuman().event["data"]["error"]).equals("WeightNotComputable");
    });

    it("Should not execute UnlockAsset", async function () {
      const xcmMessage = new XcmFragment(dotAsset)
        .withdraw_asset()
        .buy_execution()
        .unlock_asset()
        .as_v3();

      // Mock the reception of the xcm message
      await injectHrmpMessage(context, paraId, {
        type: "XcmVersionedXcm",
        payload: xcmMessage,
      } as RawXcmMessage);
      await context.createBlock();

      // Search for WeightNotComputable error
      const events = (await context.polkadotApi.query.system.events()).filter(({ event }) =>
        context.polkadotApi.events.xcmpQueue.Fail.is(event)
      );
      expect(events).to.have.lengthOf(1);
      expect(events[0].toHuman().event["data"]["error"]).equals("WeightNotComputable");
    });

    it("Should not execute NoteUnlockable", async function () {
      const xcmMessage = new XcmFragment(dotAsset)
        .withdraw_asset()
        .buy_execution()
        .note_unlockable()
        .as_v3();

      // Mock the reception of the xcm message
      await injectHrmpMessage(context, paraId, {
        type: "XcmVersionedXcm",
        payload: xcmMessage,
      } as RawXcmMessage);
      await context.createBlock();

      // Search for WeightNotComputable error
      const events = (await context.polkadotApi.query.system.events()).filter(({ event }) =>
        context.polkadotApi.events.xcmpQueue.Fail.is(event)
      );
      expect(events).to.have.lengthOf(1);
      expect(events[0].toHuman().event["data"]["error"]).equals("WeightNotComputable");
    });

    it("Should not execute RequestUnlock", async function () {
      const xcmMessage = new XcmFragment(dotAsset)
        .withdraw_asset()
        .buy_execution()
        .request_unlock()
        .as_v3();

      // Mock the reception of the xcm message
      await injectHrmpMessage(context, paraId, {
        type: "XcmVersionedXcm",
        payload: xcmMessage,
      } as RawXcmMessage);
      await context.createBlock();

      // Search for WeightNotComputable error
      const events = (await context.polkadotApi.query.system.events()).filter(({ event }) =>
        context.polkadotApi.events.xcmpQueue.Fail.is(event)
      );
      expect(events).to.have.lengthOf(1);
      expect(events[0].toHuman().event["data"]["error"]).equals("WeightNotComputable");
    });

    it("Should not execute AliasOrigin", async function () {
      const xcmMessage = new XcmFragment(dotAsset)
        .withdraw_asset()
        .buy_execution()
        .alias_origin()
        .as_v3();

      // Mock the reception of the xcm message
      await injectHrmpMessage(context, paraId, {
        type: "XcmVersionedXcm",
        payload: xcmMessage,
      } as RawXcmMessage);
      await context.createBlock();

      // Search for WeightNotComputable error
      const events = (await context.polkadotApi.query.system.events()).filter(({ event }) =>
        context.polkadotApi.events.xcmpQueue.Fail.is(event)
      );
      expect(events).to.have.lengthOf(1);
      expect(events[0].toHuman().event["data"]["error"]).equals("WeightNotComputable");
    });
  },
  "Legacy",
  "moonbase"
);

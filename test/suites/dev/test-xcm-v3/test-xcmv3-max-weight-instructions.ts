import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";

import { alith, CHARLETH_ADDRESS } from "@moonwall/util";
import {
  XcmFragment,
  RawXcmMessage,
  injectHrmpMessage,
  sovereignAccountOfSibling,
  XcmFragmentConfig,
} from "../../../helpers/xcm.js";
import { parseEther } from "ethers";

describeSuite({
  id: "D3540",
  title: "XCM V3 - Max Weight Instructions",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let dotAsset: XcmFragmentConfig;
    let amount: bigint;
    const paraId: number = 888;

    beforeAll(async () => {
      const paraSovereign = sovereignAccountOfSibling(context, paraId);
      const metadata = await context.polkadotJs().rpc.state.getMetadata();
      const balancesPalletIndex = metadata.asLatest.pallets
        .find(({ name }) => name.toString() === "Balances")!
        .index.toNumber();

      // Send some native tokens to the sovereign account of paraId (to pay fees)
      await context
        .polkadotJs()
        .tx.balances.transfer(paraSovereign, parseEther("1"))
        .signAndSend(alith);
      await context.createBlock();

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
        beneficiary: CHARLETH_ADDRESS,
      };
    });

    it({
      id: "T01",
      title: "Should not execute UniversalOrigin",
      test: async function () {
        const xcmMessage = new XcmFragment(dotAsset)
          .withdraw_asset()
          .buy_execution()
          .universal_origin({ Parachain: paraId })
          .as_v3();

        // Mock the reception of the xcm message
        await injectHrmpMessage(context, paraId, {
          type: "StagingXcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);
        await context.createBlock();

        // Search for WeightNotComputable error
        const events = (await context.polkadotJs().query.system.events()).filter(({ event }) =>
          context.polkadotJs().events.xcmpQueue.Fail.is(event)
        );
        expect(events).to.have.lengthOf(1);
        expect(events[0].event.data[2].toString()).equals("WeightNotComputable");
      },
    });

    it({
      id: "T02",
      title: "Should not execute ExportMessage",
      test: async function () {
        const xcmMessage = new XcmFragment(dotAsset)
          .withdraw_asset()
          .buy_execution()
          .export_message()
          .as_v3();

        // Mock the reception of the xcm message
        await injectHrmpMessage(context, paraId, {
          type: "StagingXcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);
        await context.createBlock();

        // Search for WeightNotComputable error
        const events = (await context.polkadotJs().query.system.events()).filter(({ event }) =>
          context.polkadotJs().events.xcmpQueue.Fail.is(event)
        );
        expect(events).to.have.lengthOf(1);
        expect(events[0].event.data[2].toString()).equals("WeightNotComputable");
      },
    });

    it({
      id: "T03",
      title: "Should not execute LockAsset",
      test: async function () {
        const xcmMessage = new XcmFragment(dotAsset)
          .withdraw_asset()
          .buy_execution()
          .lock_asset()
          .as_v3();

        // Mock the reception of the xcm message
        await injectHrmpMessage(context, paraId, {
          type: "StagingXcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);
        await context.createBlock();

        // Search for WeightNotComputable error
        const events = (await context.polkadotJs().query.system.events()).filter(({ event }) =>
          context.polkadotJs().events.xcmpQueue.Fail.is(event)
        );
        expect(events).to.have.lengthOf(1);
        expect(events[0].event.data[2].toString()).equals("WeightNotComputable");
      },
    });

    it({
      id: "T04",
      title: "Should not execute UnlockAsset",
      test: async function () {
        const xcmMessage = new XcmFragment(dotAsset)
          .withdraw_asset()
          .buy_execution()
          .unlock_asset()
          .as_v3();

        // Mock the reception of the xcm message
        await injectHrmpMessage(context, paraId, {
          type: "StagingXcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);
        await context.createBlock();

        // Search for WeightNotComputable error
        const events = (await context.polkadotJs().query.system.events()).filter(({ event }) =>
          context.polkadotJs().events.xcmpQueue.Fail.is(event)
        );
        expect(events).to.have.lengthOf(1);
        expect(events[0].event.data[2].toString()).equals("WeightNotComputable");
      },
    });

    it({
      id: "T05",
      title: "Should not execute NoteUnlockable",
      test: async function () {
        const xcmMessage = new XcmFragment(dotAsset)
          .withdraw_asset()
          .buy_execution()
          .note_unlockable()
          .as_v3();

        // Mock the reception of the xcm message
        await injectHrmpMessage(context, paraId, {
          type: "StagingXcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);
        await context.createBlock();

        // Search for WeightNotComputable error
        const events = (await context.polkadotJs().query.system.events()).filter(({ event }) =>
          context.polkadotJs().events.xcmpQueue.Fail.is(event)
        );
        expect(events).to.have.lengthOf(1);
        expect(events[0].event.data[2].toString()).equals("WeightNotComputable");
      },
    });

    it({
      id: "T06",
      title: "Should not execute RequestUnlock",
      test: async function () {
        const xcmMessage = new XcmFragment(dotAsset)
          .withdraw_asset()
          .buy_execution()
          .request_unlock()
          .as_v3();

        // Mock the reception of the xcm message
        await injectHrmpMessage(context, paraId, {
          type: "StagingXcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);
        await context.createBlock();

        // Search for WeightNotComputable error
        const events = (await context.polkadotJs().query.system.events()).filter(({ event }) =>
          context.polkadotJs().events.xcmpQueue.Fail.is(event)
        );
        expect(events).to.have.lengthOf(1);
        expect(events[0].event.data[2].toString()).equals("WeightNotComputable");
      },
    });

    it({
      id: "T07",
      title: "Should not execute AliasOrigin",
      test: async function () {
        const xcmMessage = new XcmFragment(dotAsset)
          .withdraw_asset()
          .buy_execution()
          .alias_origin()
          .as_v3();

        // Mock the reception of the xcm message
        await injectHrmpMessage(context, paraId, {
          type: "StagingXcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);
        await context.createBlock();

        // Search for WeightNotComputable error
        const events = (await context.polkadotJs().query.system.events()).filter(({ event }) =>
          context.polkadotJs().events.xcmpQueue.Fail.is(event)
        );
        expect(events).to.have.lengthOf(1);
        expect(events[0].event.data[2].toString()).equals("WeightNotComputable");
      },
    });
  },
});

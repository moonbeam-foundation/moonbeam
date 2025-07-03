import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";

import { alith, CHARLETH_ADDRESS } from "@moonwall/util";
import {
  XcmFragment,
  type RawXcmMessage,
  sovereignAccountOfSibling,
  type XcmFragmentConfig,
  injectHrmpMessageAndSeal,
} from "../../../../helpers";
import { parseEther } from "ethers";
import type { ApiPromise } from "@polkadot/api";

describeSuite({
  id: "D024038",
  title: "XCM V3 - Max Weight Instructions",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let dotAsset: XcmFragmentConfig;
    let amount: bigint;
    const paraId: number = 888;
    let api: ApiPromise;

    beforeAll(async () => {
      api = await context.polkadotJs();

      const paraSovereign = sovereignAccountOfSibling(context, paraId);
      const metadata = await api.rpc.state.getMetadata();
      const balancesPalletIndex = metadata.asLatest.pallets
        .find(({ name }) => name.toString() === "Balances")!
        .index.toNumber();

      // Send some native tokens to the sovereign account of paraId (to pay fees)
      await api.tx.balances.transferAllowDeath(paraSovereign, parseEther("1")).signAndSend(alith);
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
      title: "Should not execute ExportMessage",
      test: async function () {
        const xcmMessage = new XcmFragment(dotAsset)
          .withdraw_asset()
          .buy_execution()
          .export_message()
          .as_v3();

        // Mock the reception of the xcm message
        await injectHrmpMessageAndSeal(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        // Search for WeightNotComputable error
        const events = (await api.query.system.events())
          .filter(({ event }) => api.events.messageQueue.ProcessingFailed.is(event))
          .map((e) => e.event.data.toHuman() as { error: string });

        expect(events).to.have.lengthOf(1);
        expect(events[0].error).equals("Unsupported");
      },
    });

    it({
      id: "T02",
      title: "Should not execute LockAsset",
      test: async function () {
        const xcmMessage = new XcmFragment(dotAsset)
          .withdraw_asset()
          .buy_execution()
          .lock_asset()
          .as_v3();

        // Mock the reception of the xcm message
        await injectHrmpMessageAndSeal(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        // Search for WeightNotComputable error
        const events = (await api.query.system.events())
          .filter(({ event }) => api.events.messageQueue.ProcessingFailed.is(event))
          .map((e) => e.event.data.toHuman() as { error: string });

        expect(events).to.have.lengthOf(1);
        expect(events[0].error).equals("Unsupported");
      },
    });

    it({
      id: "T03",
      title: "Should not execute UnlockAsset",
      test: async function () {
        const xcmMessage = new XcmFragment(dotAsset)
          .withdraw_asset()
          .buy_execution()
          .unlock_asset()
          .as_v3();

        // Mock the reception of the xcm message
        await injectHrmpMessageAndSeal(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        // Search for WeightNotComputable error
        const events = (await api.query.system.events())
          .filter(({ event }) => api.events.messageQueue.ProcessingFailed.is(event))
          .map((e) => e.event.data.toHuman() as { error: string });

        expect(events).to.have.lengthOf(1);
        expect(events[0].error).equals("Unsupported");
      },
    });

    it({
      id: "T04",
      title: "Should not execute NoteUnlockable",
      test: async function () {
        const xcmMessage = new XcmFragment(dotAsset)
          .withdraw_asset()
          .buy_execution()
          .note_unlockable()
          .as_v3();

        // Mock the reception of the xcm message
        await injectHrmpMessageAndSeal(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        // Search for WeightNotComputable error
        const events = (await api.query.system.events())
          .filter(({ event }) => api.events.messageQueue.ProcessingFailed.is(event))
          .map((e) => e.event.data.toHuman() as { error: string });

        expect(events).to.have.lengthOf(1);
        expect(events[0].error).equals("Unsupported");
      },
    });

    it({
      id: "T05",
      title: "Should not execute RequestUnlock",
      test: async function () {
        const xcmMessage = new XcmFragment(dotAsset)
          .withdraw_asset()
          .buy_execution()
          .request_unlock()
          .as_v3();

        // Mock the reception of the xcm message
        await injectHrmpMessageAndSeal(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        // Search for WeightNotComputable error
        const events = (await api.query.system.events())
          .filter(({ event }) => api.events.messageQueue.ProcessingFailed.is(event))
          .map((e) => e.event.data.toHuman() as { error: string });

        expect(events).to.have.lengthOf(1);
        expect(events[0].error).equals("Unsupported");
      },
    });

    it({
      id: "T06",
      title: "Should not execute AliasOrigin",
      test: async function () {
        const xcmMessage = new XcmFragment(dotAsset)
          .withdraw_asset()
          .buy_execution()
          .alias_origin()
          .as_v3();

        // Mock the reception of the xcm message
        await injectHrmpMessageAndSeal(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        // Search for WeightNotComputable error
        const events = (await api.query.system.events())
          .filter(({ event }) => api.events.messageQueue.ProcessingFailed.is(event))
          .map((e) => e.event.data.toHuman() as { error: string });

        expect(events).to.have.lengthOf(1);
        expect(events[0].error).equals("Unsupported");
      },
    });
  },
});

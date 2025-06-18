import "@moonbeam-network/api-augment/moonbase";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";

import { alith, CHARLETH_ADDRESS } from "@moonwall/util";
import {
  XcmFragment,
  type RawXcmMessage,
  sovereignAccountOfSibling,
  type XcmFragmentConfig,
  injectHrmpMessageAndSeal,
} from "../../../../helpers/xcm.js";
import { parseEther } from "ethers";
import type { ApiPromise } from "@polkadot/api";

// Here we are testing each allowed instruction to be executed. Even if some of them throw an error,
// the important thing (and what we are testing) is that they are
// executed and are not blocked with 'WeightNotComputable' due to using max weight.
describeSuite({
  id: "D024039",
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
      title: "Should execute BurnAsset",
      test: async function () {
        const xcmMessage = new XcmFragment(dotAsset)
          .withdraw_asset()
          .buy_execution()
          .burn_asset()
          .as_v3();

        // Mock the reception of the xcm message
        await injectHrmpMessageAndSeal(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        // Search for Success
        const events = (await api.query.system.events())
          .filter(({ event }) => api.events.messageQueue.Processed.is(event))
          .map((e) => e.event.data.toHuman() as { success: boolean })
          .filter(({ success }) => success);

        expect(events).to.have.lengthOf(1);
      },
    });

    it({
      id: "T02",
      title: "Should execute ClearTopic",
      test: async function () {
        const xcmMessage = new XcmFragment(dotAsset)
          .withdraw_asset()
          .buy_execution()
          .clear_topic()
          .as_v3();

        // Mock the reception of the xcm message
        await injectHrmpMessageAndSeal(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        // Search for Success
        const events = (await api.query.system.events())
          .filter(({ event }) => api.events.messageQueue.Processed.is(event))
          .map((e) => e.event.data.toHuman() as { success: boolean })
          .filter(({ success }) => success);

        expect(events).to.have.lengthOf(1);
      },
    });

    it({
      id: "T03",
      title: "Should execute ExpectTransactStatus",
      test: async function () {
        const xcmMessage = new XcmFragment(dotAsset)
          .withdraw_asset()
          .buy_execution()
          .expect_transact_status()
          .as_v3();

        // Mock the reception of the xcm message
        await injectHrmpMessageAndSeal(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        // Search for Success
        const events = (await api.query.system.events())
          .filter(({ event }) => api.events.messageQueue.Processed.is(event))
          .map((e) => e.event.data.toHuman() as { success: boolean })
          .filter(({ success }) => success);

        expect(events).to.have.lengthOf(1);
      },
    });

    it({
      id: "T04",
      title: "Should execute ClearTransactStatus",
      test: async function () {
        const xcmMessage = new XcmFragment(dotAsset)
          .withdraw_asset()
          .buy_execution()
          .clear_transact_status()
          .as_v3();

        // Mock the reception of the xcm message
        await injectHrmpMessageAndSeal(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        // Search for Success
        const events = (await api.query.system.events())
          .filter(({ event }) => api.events.messageQueue.Processed.is(event))
          .map((e) => e.event.data.toHuman() as { success: boolean })
          .filter(({ success }) => success);

        expect(events).to.have.lengthOf(1);
      },
    });

    it({
      id: "T05",
      title: "Should execute SetFeesMode",
      test: async function () {
        const xcmMessage = new XcmFragment(dotAsset)
          .withdraw_asset()
          .buy_execution()
          .set_fees_mode()
          .as_v3();

        // Mock the reception of the xcm message
        await injectHrmpMessageAndSeal(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        // Search for Success
        const events = (await api.query.system.events())
          .filter(({ event }) => api.events.messageQueue.Processed.is(event))
          .map((e) => e.event.data.toHuman() as { success: boolean })
          .filter(({ success }) => success);

        expect(events).to.have.lengthOf(1);
      },
    });

    it({
      id: "T06",
      title: "Should execute SetTopic",
      test: async function () {
        const xcmMessage = new XcmFragment(dotAsset)
          .withdraw_asset()
          .buy_execution()
          // SetTopic expects an array of 32 bytes
          .set_topic()
          .as_v3();

        // Mock the reception of the xcm message
        await injectHrmpMessageAndSeal(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        // Search for Success
        const events = (await api.query.system.events())
          .filter(({ event }) => api.events.messageQueue.Processed.is(event))
          .map((e) => e.event.data.toHuman() as { success: boolean })
          .filter(({ success }) => success);

        expect(events).to.have.lengthOf(1);
      },
    });

    it({
      id: "T07",
      title: "Should fail to execute ReportHolding (Transport)",
      test: async function () {
        const xcmMessage = new XcmFragment(dotAsset)
          .withdraw_asset()
          .buy_execution()
          .report_holding()
          .as_v3();

        // Mock the reception of the xcm message
        await injectHrmpMessageAndSeal(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        // Search for failure
        const events = (await api.query.system.events())
          .filter(({ event }) => api.events.messageQueue.Processed.is(event))
          .map((e) => e.event.data.toHuman() as { success: boolean })
          .filter(({ success }) => !success);

        expect(events).to.have.lengthOf(1);
        // pallet-message-queue does not show an error when "success" is false.
        // https://github.com/paritytech/polkadot-sdk/issues/478
        // >
        // expect(events[0].event.data[2].toString()).equals("Transport");
      },
    });

    it({
      id: "T08",
      title: "Should fail to execute ExpectAsset (ExpectationFalse)",
      test: async function () {
        const xcmMessage = new XcmFragment(dotAsset)
          .withdraw_asset()
          .buy_execution()
          .expect_asset()
          .as_v3();

        // Mock the reception of the xcm message
        await injectHrmpMessageAndSeal(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        // Search for failure
        const events = (await api.query.system.events())
          .filter(({ event }) => api.events.messageQueue.Processed.is(event))
          .map((e) => e.event.data.toHuman() as { success: boolean })
          .filter(({ success }) => !success);

        expect(events).to.have.lengthOf(1);
        // pallet-message-queue does not show an error when "success" is false.
        // https://github.com/paritytech/polkadot-sdk/issues/478
        // >
        // expect(events[0].event.data[2].toString()).equals("ExpectationFalse");
      },
    });

    it({
      id: "T09",
      title: "Should fail to execute ExpectOrigin (ExpectationFalse)",
      test: async function () {
        const xcmMessage = new XcmFragment(dotAsset)
          .withdraw_asset()
          .buy_execution()
          .expect_origin()
          .as_v3();

        // Mock the reception of the xcm message
        await injectHrmpMessageAndSeal(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        // Search for failure
        const events = (await api.query.system.events())
          .filter(({ event }) => api.events.messageQueue.Processed.is(event))
          .map((e) => e.event.data.toHuman() as { success: boolean })
          .filter(({ success }) => !success);

        expect(events).to.have.lengthOf(1);
        // pallet-message-queue does not show an error when "success" is false.
        // https://github.com/paritytech/polkadot-sdk/issues/478
        // >
        // expect(events[0].event.data[2].toString()).equals("ExpectationFalse");
      },
    });

    it({
      id: "T10",
      title: "Should fail to execute ExpectError (ExpectationFalse)",
      test: async function () {
        const xcmMessage = new XcmFragment(dotAsset)
          .withdraw_asset()
          .buy_execution()
          .expect_error()
          .as_v3();

        // Mock the reception of the xcm message
        await injectHrmpMessageAndSeal(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        // Search for failure
        const events = (await api.query.system.events())
          .filter(({ event }) => api.events.messageQueue.Processed.is(event))
          .map((e) => e.event.data.toHuman() as { success: boolean })
          .filter(({ success }) => !success);

        expect(events).to.have.lengthOf(1);
        // pallet-message-queue does not show an error when "success" is false.
        // https://github.com/paritytech/polkadot-sdk/issues/478
        // >
        // expect(events[0].event.data[2].toString()).equals("ExpectationFalse");
      },
    });

    it({
      id: "T11",
      title: "Should fail to execute QueryPallet (Transport)",
      test: async function () {
        const xcmMessage = new XcmFragment(dotAsset)
          .withdraw_asset()
          .buy_execution()
          .query_pallet()
          .as_v3();

        // Mock the reception of the xcm message
        await injectHrmpMessageAndSeal(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        // Search for failure
        const events = (await api.query.system.events())
          .filter(({ event }) => api.events.messageQueue.Processed.is(event))
          .map((e) => e.event.data.toHuman() as { success: boolean })
          .filter(({ success }) => !success);

        expect(events).to.have.lengthOf(1);
        // pallet-message-queue does not show an error when "success" is false.
        // https://github.com/paritytech/polkadot-sdk/issues/478
        // >
        // expect(events[0].event.data[2].toString()).equals("Transport");
      },
    });

    it({
      id: "T12",
      title: "Should fail to execute ExpectPallet (NameMismatch)",
      test: async function () {
        const xcmMessage = new XcmFragment(dotAsset)
          .withdraw_asset()
          .buy_execution()
          .expect_pallet()
          .as_v3();

        // Mock the reception of the xcm message
        await injectHrmpMessageAndSeal(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        // Search for failure
        const events = (await api.query.system.events())
          .filter(({ event }) => api.events.messageQueue.Processed.is(event))
          .map((e) => e.event.data.toHuman() as { success: boolean })
          .filter(({ success }) => !success);

        expect(events).to.have.lengthOf(1);
        // pallet-message-queue does not show an error when "success" is false.
        // https://github.com/paritytech/polkadot-sdk/issues/478
        // >
        // expect(events[0].event.data[2].toString()).equals("NameMismatch");
      },
    });

    it({
      id: "T13",
      title: "Should fail to execute ReportTransactStatus (Transport)",
      test: async function () {
        const xcmMessage = new XcmFragment(dotAsset)
          .withdraw_asset()
          .buy_execution()
          .report_transact_status()
          .as_v3();

        // Mock the reception of the xcm message
        await injectHrmpMessageAndSeal(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        // Search for failure
        const events = (await api.query.system.events())
          .filter(({ event }) => api.events.messageQueue.Processed.is(event))
          .map((e) => e.event.data.toHuman() as { success: boolean })
          .filter(({ success }) => !success);

        expect(events).to.have.lengthOf(1);
        // pallet-message-queue does not show an error when "success" is false.
        // https://github.com/paritytech/polkadot-sdk/issues/478
        // >
        // expect(events[0].event.data[2].toString()).equals("Transport");
      },
    });

    it({
      id: "T14",
      title: "Should fail to execute UnpaidExecution (BadOrigin)",
      test: async function () {
        const xcmMessage = new XcmFragment(dotAsset)
          .withdraw_asset()
          .buy_execution()
          .unpaid_execution()
          .as_v3();

        // Mock the reception of the xcm message
        await injectHrmpMessageAndSeal(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        // Search for failure
        const events = (await api.query.system.events())
          .filter(({ event }) => api.events.messageQueue.Processed.is(event))
          .map((e) => e.event.data.toHuman() as { success: boolean })
          .filter(({ success }) => !success);

        expect(events).to.have.lengthOf(1);
        // pallet-message-queue does not show an error when "success" is false.
        // https://github.com/paritytech/polkadot-sdk/issues/478
        // >
        // expect(events[0].event.data[2].toString()).equals("BadOrigin");
      },
    });
  },
});

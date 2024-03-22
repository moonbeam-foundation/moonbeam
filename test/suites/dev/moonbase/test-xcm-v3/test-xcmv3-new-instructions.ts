import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";

import { alith, CHARLETH_ADDRESS } from "@moonwall/util";
import {
  XcmFragment,
  RawXcmMessage,
  injectHrmpMessage,
  sovereignAccountOfSibling,
  XcmFragmentConfig,
} from "../../../../helpers/xcm.js";
import { parseEther } from "ethers";

// Here we are testing each allowed instruction to be executed. Even if some of them throw an error,
// the important thing (and what we are testing) is that they are
// executed and are not blocked with 'WeightNotComputable' due to using max weight.
describeSuite({
  id: "D014140",
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
        .tx.balances.transferAllowDeath(paraSovereign, parseEther("1"))
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
      title: "Should execute BurnAsset",
      test: async function () {
        const xcmMessage = new XcmFragment(dotAsset)
          .withdraw_asset()
          .buy_execution()
          .burn_asset()
          .as_v3();

        // Mock the reception of the xcm message
        await injectHrmpMessage(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);
        await context.createBlock();

        // Search for Success
        const events = (await context.polkadotJs().query.system.events()).filter(({ event }) =>
          context.polkadotJs().events.xcmpQueue.Success.is(event)
        );
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
        await injectHrmpMessage(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);
        await context.createBlock();

        // Search for Success
        const events = (await context.polkadotJs().query.system.events()).filter(({ event }) =>
          context.polkadotJs().events.xcmpQueue.Success.is(event)
        );
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
        await injectHrmpMessage(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);
        await context.createBlock();

        // Search for Success
        const events = (await context.polkadotJs().query.system.events()).filter(({ event }) =>
          context.polkadotJs().events.xcmpQueue.Success.is(event)
        );
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
        await injectHrmpMessage(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);
        await context.createBlock();

        // Search for Success
        const events = (await context.polkadotJs().query.system.events()).filter(({ event }) =>
          context.polkadotJs().events.xcmpQueue.Success.is(event)
        );
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
        await injectHrmpMessage(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);
        await context.createBlock();

        // Search for Success
        const events = (await context.polkadotJs().query.system.events()).filter(({ event }) =>
          context.polkadotJs().events.xcmpQueue.Success.is(event)
        );
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
        await injectHrmpMessage(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);
        await context.createBlock();

        // Search for Success
        const events = (await context.polkadotJs().query.system.events()).filter(({ event }) =>
          context.polkadotJs().events.xcmpQueue.Success.is(event)
        );
        expect(events).to.have.lengthOf(1);
      },
    });

    it({
      id: "T07",
      title: "Should execute ReportHolding (Transport)",
      test: async function () {
        const xcmMessage = new XcmFragment(dotAsset)
          .withdraw_asset()
          .buy_execution()
          .report_holding()
          .as_v3();

        // Mock the reception of the xcm message
        await injectHrmpMessage(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);
        await context.createBlock();

        // Search for Success
        const events = (await context.polkadotJs().query.system.events()).filter(({ event }) =>
          context.polkadotJs().events.xcmpQueue.Fail.is(event)
        );
        expect(events).to.have.lengthOf(1);
        expect(events[0].event.data[2].toString()).equals("Transport");
      },
    });

    it({
      id: "T08",
      title: "Should execute ExpectAsset (ExpectationFalse)",
      test: async function () {
        const xcmMessage = new XcmFragment(dotAsset)
          .withdraw_asset()
          .buy_execution()
          .expect_asset()
          .as_v3();

        // Mock the reception of the xcm message
        await injectHrmpMessage(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);
        await context.createBlock();

        // Search for Success
        const events = (await context.polkadotJs().query.system.events()).filter(({ event }) =>
          context.polkadotJs().events.xcmpQueue.Fail.is(event)
        );
        expect(events).to.have.lengthOf(1);
        expect(events[0].event.data[2].toString()).equals("ExpectationFalse");
      },
    });

    it({
      id: "T09",
      title: "Should execute ExpectOrigin (ExpectationFalse)",
      test: async function () {
        const xcmMessage = new XcmFragment(dotAsset)
          .withdraw_asset()
          .buy_execution()
          .expect_origin()
          .as_v3();

        // Mock the reception of the xcm message
        await injectHrmpMessage(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);
        await context.createBlock();

        // Search for Success
        const events = (await context.polkadotJs().query.system.events()).filter(({ event }) =>
          context.polkadotJs().events.xcmpQueue.Fail.is(event)
        );
        expect(events).to.have.lengthOf(1);
        expect(events[0].event.data[2].toString()).equals("ExpectationFalse");
      },
    });

    it({
      id: "T10",
      title: "Should execute ExpectError (ExpectationFalse)",
      test: async function () {
        const xcmMessage = new XcmFragment(dotAsset)
          .withdraw_asset()
          .buy_execution()
          .expect_error()
          .as_v3();

        // Mock the reception of the xcm message
        await injectHrmpMessage(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);
        await context.createBlock();

        // Search for Success
        const events = (await context.polkadotJs().query.system.events()).filter(({ event }) =>
          context.polkadotJs().events.xcmpQueue.Fail.is(event)
        );
        expect(events).to.have.lengthOf(1);
        expect(events[0].event.data[2].toString()).equals("ExpectationFalse");
      },
    });

    it({
      id: "T11",
      title: "Should execute QueryPallet (Transport)",
      test: async function () {
        const xcmMessage = new XcmFragment(dotAsset)
          .withdraw_asset()
          .buy_execution()
          .query_pallet()
          .as_v3();

        // Mock the reception of the xcm message
        await injectHrmpMessage(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);
        await context.createBlock();

        // Search for Success
        const events = (await context.polkadotJs().query.system.events()).filter(({ event }) =>
          context.polkadotJs().events.xcmpQueue.Fail.is(event)
        );
        expect(events).to.have.lengthOf(1);
        expect(events[0].event.data[2].toString()).equals("Transport");
      },
    });

    it({
      id: "T12",
      title: "Should execute ExpectPallet (NameMismatch)",
      test: async function () {
        const xcmMessage = new XcmFragment(dotAsset)
          .withdraw_asset()
          .buy_execution()
          .expect_pallet()
          .as_v3();

        // Mock the reception of the xcm message
        await injectHrmpMessage(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);
        await context.createBlock();

        // Search for Success
        const events = (await context.polkadotJs().query.system.events()).filter(({ event }) =>
          context.polkadotJs().events.xcmpQueue.Fail.is(event)
        );
        expect(events).to.have.lengthOf(1);
        expect(events[0].event.data[2].toString()).equals("NameMismatch");
      },
    });

    it({
      id: "T13",
      title: "Should execute ReportTransactStatus (Transport)",
      test: async function () {
        const xcmMessage = new XcmFragment(dotAsset)
          .withdraw_asset()
          .buy_execution()
          .report_transact_status()
          .as_v3();

        // Mock the reception of the xcm message
        await injectHrmpMessage(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);
        await context.createBlock();

        // Search for Success
        const events = (await context.polkadotJs().query.system.events()).filter(({ event }) =>
          context.polkadotJs().events.xcmpQueue.Fail.is(event)
        );
        expect(events).to.have.lengthOf(1);
        expect(events[0].event.data[2].toString()).equals("Transport");
      },
    });

    it({
      id: "T14",
      title: "Should execute UnpaidExecution (BadOrigin)",
      test: async function () {
        const xcmMessage = new XcmFragment(dotAsset)
          .withdraw_asset()
          .buy_execution()
          .unpaid_execution()
          .as_v3();

        // Mock the reception of the xcm message
        await injectHrmpMessage(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);
        await context.createBlock();

        // Search for Success
        const events = (await context.polkadotJs().query.system.events()).filter(({ event }) =>
          context.polkadotJs().events.xcmpQueue.Fail.is(event)
        );
        expect(events).to.have.lengthOf(1);
        expect(events[0].event.data[2].toString()).equals("BadOrigin");
      },
    });
  },
});

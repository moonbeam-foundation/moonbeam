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

// Here we are testing each allowed instruction to be executed. Even if some of them throw an error,
// the important thing (and what we are testing) is that they are
// executed and are not blocked with 'WeightNotComputable' due to using max weight.
describeDevMoonbeam(
  "XCM V3 Instructions",
  (context) => {
    let dotAsset: any;
    let amount: bigint;
    let paraId: number;

    before("Set up initial constants", async function () {
      paraId = 888;
      const paraSovereign = sovereignAccountOfSibling(context, paraId);
      const metadata = await context.polkadotApi.rpc.state.getMetadata();
      const balancesPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find(
        (pallet) => pallet.name === "Balances"
      ).index;

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

    it("Should execute BurnAsset", async function () {
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
      const records = (await context.polkadotApi.query.system.events()) as any;
      const events = records.filter(
        ({ event }) => event.section == "xcmpQueue" && event.method == "Success"
      );
      expect(events).to.have.lengthOf(1);
    });

    it("Should execute ClearTopic", async function () {
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
      const records = (await context.polkadotApi.query.system.events()) as any;
      const events = records.filter(
        ({ event }) => event.section == "xcmpQueue" && event.method == "Success"
      );
      expect(events).to.have.lengthOf(1);
    });

    it("Should execute ExpectTransactStatus", async function () {
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
      const records = (await context.polkadotApi.query.system.events()) as any;
      const events = records.filter(
        ({ event }) => event.section == "xcmpQueue" && event.method == "Success"
      );
      expect(events).to.have.lengthOf(1);
    });

    it("Should execute ClearTransactStatus", async function () {
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

      // Search for Transport error
      const records = (await context.polkadotApi.query.system.events()) as any;
      const events = records.filter(
        ({ event }) => event.section == "xcmpQueue" && event.method == "Success"
      );
      expect(events).to.have.lengthOf(1);
    });

    it("Should execute SetFeesMode", async function () {
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
      const records = (await context.polkadotApi.query.system.events()) as any;
      const events = records.filter(
        ({ event }) => event.section == "xcmpQueue" && event.method == "Success"
      );
      expect(events).to.have.lengthOf(1);
    });

    it("Should execute SetTopic", async function () {
      const xcmMessage = new XcmFragment(dotAsset)
        .withdraw_asset()
        .buy_execution()
        // SetTopic expects an array of 32 bytes
        .set_topic([
          122, 22, 113, 160, 34, 76, 137, 39, 176, 143, 151, 128, 39, 213, 134, 171, 104, 104, 222,
          13, 49, 187, 91, 201, 86, 182, 37, 206, 210, 171, 24, 196,
        ])
        .as_v3();

      // Mock the reception of the xcm message
      await injectHrmpMessage(context, paraId, {
        type: "XcmVersionedXcm",
        payload: xcmMessage,
      } as RawXcmMessage);
      await context.createBlock();

      // Search for UnknownClaim error
      const records = (await context.polkadotApi.query.system.events()) as any;
      const events = records.filter(
        ({ event }) => event.section == "xcmpQueue" && event.method == "Success"
      );
      expect(events).to.have.lengthOf(1);
    });

    it("Should execute ReportHolding (Transport error)", async function () {
      const xcmMessage = new XcmFragment(dotAsset)
        .withdraw_asset()
        .buy_execution()
        .report_holding(1000)
        .as_v3();

      // Mock the reception of the xcm message
      await injectHrmpMessage(context, paraId, {
        type: "XcmVersionedXcm",
        payload: xcmMessage,
      } as RawXcmMessage);
      await context.createBlock();

      const records = (await context.polkadotApi.query.system.events()) as any;
      const events = records.filter(
        ({ event }) => event.section == "xcmpQueue" && event.method == "Fail"
      );
      expect(events).to.have.lengthOf(1);
      expect(events[0].toHuman().event.data.error).equals("Transport");
    });

    it("Should execute ExpectAsset (ExpectationFalse)", async function () {
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

      // Search for ExpectationFalse error
      const records = (await context.polkadotApi.query.system.events()) as any;
      const events = records.filter(
        ({ event }) => event.section == "xcmpQueue" && event.method == "Fail"
      );
      expect(events).to.have.lengthOf(1);
      expect(events[0].toHuman().event.data.error).equals("ExpectationFalse");
    });

    it("Should execute ExpectOrigin (ExpectationFalse)", async function () {
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

      // Search for ExpectationFalse error
      const records = (await context.polkadotApi.query.system.events()) as any;
      const events = records.filter(
        ({ event }) => event.section == "xcmpQueue" && event.method == "Fail"
      );
      expect(events).to.have.lengthOf(1);
      expect(events[0].toHuman().event.data.error).equals("ExpectationFalse");
    });

    it("Should execute ExpectError (ExpectationFalse)", async function () {
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

      // Search for ExpectationFalse error
      const records = (await context.polkadotApi.query.system.events()) as any;
      const events = records.filter(
        ({ event }) => event.section == "xcmpQueue" && event.method == "Fail"
      );
      expect(events).to.have.lengthOf(1);
      expect(events[0].toHuman().event.data.error).equals("ExpectationFalse");
    });

    it("Should execute QueryPallet (Unroutable)", async function () {
      const xcmMessage = new XcmFragment(dotAsset)
        .withdraw_asset()
        .buy_execution()
        .query_pallet(1002)
        .as_v3();

      // Mock the reception of the xcm message
      await injectHrmpMessage(context, paraId, {
        type: "XcmVersionedXcm",
        payload: xcmMessage,
      } as RawXcmMessage);
      await context.createBlock();

      // Search for Unroutable error
      const records = (await context.polkadotApi.query.system.events()) as any;
      const events = records.filter(
        ({ event }) => event.section == "xcmpQueue" && event.method == "Fail"
      );
      expect(events).to.have.lengthOf(1);
      expect(events[0].toHuman().event.data.error).equals("Unroutable");
    });

    it("Should execute ExpectPallet (NameMismatch)", async function () {
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

      // Search for NameMismatch error
      const records = (await context.polkadotApi.query.system.events()) as any;
      const events = records.filter(
        ({ event }) => event.section == "xcmpQueue" && event.method == "Fail"
      );
      expect(events).to.have.lengthOf(1);
      expect(events[0].toHuman().event.data.error).equals("NameMismatch");
    });

    it("Should execute ReportTransactStatus (Transport error)", async function () {
      const xcmMessage = new XcmFragment(dotAsset)
        .withdraw_asset()
        .buy_execution()
        .report_transact_status(1000)
        .as_v3();

      // Mock the reception of the xcm message
      await injectHrmpMessage(context, paraId, {
        type: "XcmVersionedXcm",
        payload: xcmMessage,
      } as RawXcmMessage);
      await context.createBlock();

      // Search for Transport error
      const records = (await context.polkadotApi.query.system.events()) as any;
      const events = records.filter(
        ({ event }) => event.section == "xcmpQueue" && event.method == "Fail"
      );
      expect(events).to.have.lengthOf(1);
      expect(events[0].toHuman().event.data.error).equals("Transport");
    });

    it("Should execute UnpaidExecution (BadOrigin)", async function () {
      const xcmMessage = new XcmFragment(dotAsset)
        .withdraw_asset()
        .buy_execution()
        .unpaid_execution(1)
        .as_v3();

      // Mock the reception of the xcm message
      await injectHrmpMessage(context, paraId, {
        type: "XcmVersionedXcm",
        payload: xcmMessage,
      } as RawXcmMessage);
      await context.createBlock();

      // Search for BadOrigin error
      const records = (await context.polkadotApi.query.system.events()) as any;
      const events = records.filter(
        ({ event }) => event.section == "xcmpQueue" && event.method == "Fail"
      );
      expect(events).to.have.lengthOf(1);
      expect(events[0].toHuman().event.data.error).equals("BadOrigin");
    });
  },
  "Legacy",
  "moonbase"
);

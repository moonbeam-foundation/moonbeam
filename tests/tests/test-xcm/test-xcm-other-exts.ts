import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { alith, generateKeyringPair, CHARLETH_ADDRESS } from "../../util/accounts";
import { XcmVersionedXcm, XcmV3Instruction, XcmV3TraitsOutcome } from "@polkadot/types/lookup";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { expectOk, expectSubstrateEvent, expectSuccessfulXCM } from "../../util/expect";
import { GLMR } from "../../util/constants";
import { XcmFragment, injectHrmpMessage, RawXcmMessage, sovereignAccountOfSibling } from "../../util/xcm";
import { BN } from "@polkadot/util";
import { expectEVMResult } from "../../util/eth-transactions";
import {
  ALITH_TRANSACTION_TEMPLATE,
  createTransaction,
} from "../../util/transactions";

const foreign_para_id = 2000;

const MAX_WEIGHT = {
  refTime: 20_000_000_000,
  proofSize: 10000,
};

describeDevMoonbeam(
  "XCM V3 Instructions",
  (context) => {
    let balancesPalletIndex: number;
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

    it.skip("Should execute ReportHolding", async function () {
      const xcmMessage = new XcmFragment(dotAsset)
        .withdraw_asset()
        .buy_execution()
        .deposit_asset()
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

    it.skip("Should execute ExpectAsset", async function () {
      const xcmMessage = new XcmFragment(dotAsset)
        .withdraw_asset()
        .buy_execution()
        .expect_asset()
        .as_v3();

      const blockResponse = await context.createBlock(
        context.polkadotApi.tx.polkadotXcm.execute(xcmMessage, MAX_WEIGHT)
      );
      const { data } = expectSubstrateEvent(blockResponse, "polkadotXcm", "Attempted");
      expectSuccessfulXCM(data[0] as XcmV3TraitsOutcome);
    });

    it.skip("Should execute ExpectOrigin (ExpectationFalse)", async function () {
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

    it.skip("Should execute ExpectError (ExpectationFalse)", async function () {
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

    it.skip("Should execute ExpectTransactStatus", async function () {
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

    it.skip("TODO: CHECK Should execute QueryPallet", async function () {
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

      // Search for Success
      const records = (await context.polkadotApi.query.system.events()) as any;
      const events = records.filter(
        ({ event }) => event.section == "xcmpQueue" && event.method == "Success"
      );
      expect(events).to.have.lengthOf(1);
    });

    it.skip("Should execute ExpectPallet (NameMismatch)", async function () {
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

    it.skip("TODO: CHECK Should execute ReportTransactStatus (Transport)", async function () {
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

    it.skip("Should execute SetFeesMode", async function () {
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

    it.skip("Should execute SetTopic", async function () {
      const xcmMessage = new XcmFragment(dotAsset)
        .withdraw_asset()
        .buy_execution()
        // SetTopic expects an array of 32 bytes
        .set_topic(Uint8Array.from([
          122,  22, 113, 160,  34,  76, 137,  39,
          176, 143, 151, 128,  39, 213, 134, 171,
          104, 104, 222,  13,  49, 187,  91, 201,
           86, 182,  37, 206, 210, 171,  24, 196
        ]))
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

    /*   const blockResponse = await context.createBlock(
        context.polkadotApi.tx.polkadotXcm.execute(xcmMessage, MAX_WEIGHT)
      ); */

      /* const { data } = expectSubstrateEvent(blockResponse, "polkadotXcm", "Attempted");
      expectSuccessfulXCM(data[0] as XcmV3TraitsOutcome); */
    });

    it.skip("Should execute ClearTopic", async function () {
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

    it.skip("Should execute UnpaidExecution (BadOrigin)", async function () {
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

    it.skip("Should execute BurnAsset", async function () {
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

    // WEIGHT::MAX instructions
    it.skip("Should not execute UniversalOrigin", async function () {
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
      const records = (await context.polkadotApi.query.system.events()) as any;
      const events = records.filter(
        ({ event }) => event.section == "xcmpQueue" && event.method == "Fail"
      );
      expect(events).to.have.lengthOf(1);
      expect(events[0].toHuman().event.data.error).equals("WeightNotComputable");
    });

    it.skip("Should not execute ExportMessage", async function () {
      const xcmMessage = new XcmFragment(dotAsset)
        .withdraw_asset()
        .buy_execution()
        .export_message("Ethereum", 1, [1,2,3])
        .as_v3();
      
      // Mock the reception of the xcm message
      await injectHrmpMessage(context, paraId, {
        type: "XcmVersionedXcm",
        payload: xcmMessage,
      } as RawXcmMessage);
      await context.createBlock();

      // Search for WeightNotComputable error
      const records = (await context.polkadotApi.query.system.events()) as any;
      const events = records.filter(
        ({ event }) => event.section == "xcmpQueue" && event.method == "Fail"
      );
      expect(events).to.have.lengthOf(1);
      expect(events[0].toHuman().event.data.error).equals("WeightNotComputable");
    });

    it.skip("Should not execute LockAsset", async function () {
      const xcmMessage = new XcmFragment(dotAsset)
        .withdraw_asset()
        .buy_execution()
        .lock_asset(0, 1)
        .as_v3();
      
      // Mock the reception of the xcm message
      await injectHrmpMessage(context, paraId, {
        type: "XcmVersionedXcm",
        payload: xcmMessage,
      } as RawXcmMessage);
      await context.createBlock();

      // Search for WeightNotComputable error
      const records = (await context.polkadotApi.query.system.events()) as any;
      const events = records.filter(
        ({ event }) => event.section == "xcmpQueue" && event.method == "Fail"
      );
      expect(events).to.have.lengthOf(1);
      expect(events[0].toHuman().event.data.error).equals("WeightNotComputable");
    });

    it.skip("Should not execute UnlockAsset", async function () {
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
      const records = (await context.polkadotApi.query.system.events()) as any;
      const events = records.filter(
        ({ event }) => event.section == "xcmpQueue" && event.method == "Fail"
      );
      expect(events).to.have.lengthOf(1);
      expect(events[0].toHuman().event.data.error).equals("WeightNotComputable");
    });

    it.skip("Should not execute NoteUnlockable", async function () {
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
      const records = (await context.polkadotApi.query.system.events()) as any;
      const events = records.filter(
        ({ event }) => event.section == "xcmpQueue" && event.method == "Fail"
      );
      expect(events).to.have.lengthOf(1);
      expect(events[0].toHuman().event.data.error).equals("WeightNotComputable");
    });

    it.skip("Should not execute RequestUnlock", async function () {
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
      const records = (await context.polkadotApi.query.system.events()) as any;
      const events = records.filter(
        ({ event }) => event.section == "xcmpQueue" && event.method == "Fail"
      );
      expect(events).to.have.lengthOf(1);
      expect(events[0].toHuman().event.data.error).equals("WeightNotComputable");
    });

    it.skip("Should not execute AliasOrigin", async function () {
      const xcmMessage = new XcmFragment(dotAsset)
        .withdraw_asset()
        .buy_execution()
        .alias_origin(1)
        .as_v3();
      
      // Mock the reception of the xcm message
      await injectHrmpMessage(context, paraId, {
        type: "XcmVersionedXcm",
        payload: xcmMessage,
      } as RawXcmMessage);
      await context.createBlock();

      // Search for WeightNotComputable error
      const records = (await context.polkadotApi.query.system.events()) as any;
      const events = records.filter(
        ({ event }) => event.section == "xcmpQueue" && event.method == "Fail"
      );
      expect(events).to.have.lengthOf(1);
      expect(events[0].toHuman().event.data.error).equals("WeightNotComputable");
    });
  },
  "Legacy",
  "moonbase"
);

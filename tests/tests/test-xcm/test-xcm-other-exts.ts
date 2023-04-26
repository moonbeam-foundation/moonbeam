import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { alith, generateKeyringPair } from "../../util/accounts";
import { XcmVersionedXcm, XcmV3Instruction, XcmV3TraitsOutcome } from "@polkadot/types/lookup";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { expectOk, expectSubstrateEvent, expectSuccessfulXCM } from "../../util/expect";
import { GLMR } from "../../util/constants";
import { XcmFragment } from "../../util/xcm";
import { BN } from "@polkadot/util";

const foreign_para_id = 2000;

const MAX_WEIGHT = {
  refTime: 20_000_000_000,
  proofSize: 10000,
};

// TODO: Add more test case permutations
describeDevMoonbeam(
  "XCM Other Extrinsics",
  (context) => {
    let balancesPalletIndex: number;
    let dotAsset: any;
    let amount: bigint;

    before("TODO: FILL IN", async function () {
      const metadata = await context.polkadotApi.rpc.state.getMetadata();
      balancesPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find((pallet) => {
        return pallet.name === "Balances";
      }).index;
      const random = generateKeyringPair();
      amount = 100_000_000_000_000_000_000n; // 100 UNIT
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
        beneficiary: random.publicKey,
      };
    });

    it("Should execute ClearTopic", async function () {
      const message = {
        V3: [
          {
            ClearTopic: null,
          },
        ],
      };

      const blockResponse = await context.createBlock(
        context.polkadotApi.tx.polkadotXcm.execute(message, MAX_WEIGHT)
      );

      const { data } = expectSubstrateEvent(blockResponse, "polkadotXcm", "Attempted");
      expectSuccessfulXCM(data[0] as XcmV3TraitsOutcome);
    });

    it.skip("Should execute AliasOrigin", async function () {
      const message = {
        V3: [
          {
            AliasOrigin: {
              parents: 1,
              interior: {
                Here: null,
              },
            },
          },
        ],
      };

      const blockResponse = await context.createBlock(
        context.polkadotApi.tx.polkadotXcm.execute(message, MAX_WEIGHT)
      );

      const { data } = expectSubstrateEvent(blockResponse, "polkadotXcm", "Attempted");
      expectSuccessfulXCM(data[0] as XcmV3TraitsOutcome);
    });

    it("Should execute BurnAsset", async function () {
      const xcmMessage = new XcmFragment(dotAsset)
        .withdraw_asset()
        .burn_asset()
        .as_v3();

      const blockResponse = await context.createBlock(
        context.polkadotApi.tx.polkadotXcm.execute(xcmMessage, MAX_WEIGHT)
      );

      const { data } = expectSubstrateEvent(blockResponse, "polkadotXcm", "Attempted");
      expectSuccessfulXCM(data[0] as XcmV3TraitsOutcome);
    });

    // report_holding
    // burn_asset
    // expect_asset
    // expect_origin
    // expect_error
    // expect_transact_status
    // query_pallet
    // expect_pallet
    // report_transact_status
    // clear_transact_status
    // universal_origin
    // export_message
    // lock_asset
    // unlock_asset
    // note_unlockable
    // request_unlock
    // set_fees_mode
    // set_topic
    // unpaid_execution
  },
  "Legacy",
  "moonbase"
);

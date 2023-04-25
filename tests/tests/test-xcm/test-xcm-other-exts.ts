import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { alith, generateKeyringPair } from "../../util/accounts";
import { XcmVersionedXcm, XcmV3Instruction, XcmV3TraitsOutcome } from "@polkadot/types/lookup";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { expectOk, expectSubstrateEvent, expectSuccessfulXCM } from "../../util/expect";
import { GLMR } from "../../util/constants";

const foreign_para_id = 2000;

const MAX_WEIGHT = {
  refTime: 5000000000,
  proofSize: 10000,
};

// TODO: Add more test case permutations
describeDevMoonbeam(
  "XCM Other Extrinsics",
  (context) => {
    let balancesPalletIndex: number;

    before("TODO: FILL IN", async function () {
      const metadata = await context.polkadotApi.rpc.state.getMetadata();
      balancesPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find((pallet) => {
        return pallet.name === "Balances";
      }).index;
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

    it("Should execute AliasOrigin", async function () {
      let random = generateKeyringPair();

      const transferCall = context.polkadotApi.tx.balances.transfer(random.address, 1n * GLMR);
      const transferCallEncoded = transferCall?.method.toHex();

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
      let random = generateKeyringPair();

      const transferCall = context.polkadotApi.tx.balances.transfer(random.address, 1n * GLMR);
      const transferCallEncoded = transferCall?.method.toHex();

      const message = {
        V3: [
          /// TODO: add all other XCM bits required to make this a reasonable test like buy execution withdraw asset etc
          {
            BurnAsset: [
              {
                Concrete: {
                  parents: 1,
                  interior: {
                    X1: { PalletInstance: balancesPalletIndex },
                  },
                },
                fungible: 1n,
              },
            ],
          },
        ],
      };

      const blockResponse = await context.createBlock(
        context.polkadotApi.tx.polkadotXcm.execute(message, MAX_WEIGHT)
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

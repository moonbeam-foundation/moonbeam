import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";

import { alith } from "@moonwall/util";

import {
  XcmFragment,
  type RawXcmMessage,
  injectHrmpMessageAndSeal,
} from "../../../../helpers/xcm.js";
import { registerOldForeignAsset } from "../../../../helpers/assets.js";

const FOREIGN_TOKEN = 1_000_000_000_000n;

const palletId = "0x6D6f646c617373746d6E67720000000000000000";
const statemint_para_id = 1001;
const statemint_assets_pallet_instance = 50;

const assetMetadata = {
  name: "FOREIGN",
  symbol: "FOREIGN",
  decimals: 12n,
  isFrozen: false,
};
const STATEMINT_LOCATION = {
  Xcm: {
    parents: 1,
    interior: {
      X3: [
        { Parachain: statemint_para_id },
        { PalletInstance: statemint_assets_pallet_instance },
        { GeneralIndex: 0 },
      ],
    },
  },
};
const STATEMINT_ASSET_ONE_LOCATION = {
  Xcm: {
    parents: 1,
    interior: {
      X3: [
        { Parachain: statemint_para_id },
        { PalletInstance: statemint_assets_pallet_instance },
        { GeneralIndex: 1 },
      ],
    },
  },
};

describeSuite({
  id: "D024212",
  title: "Mock XCM - receive horizontal transfer",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let assetIdZero: string;
    let assetIdOne: string;

    beforeAll(async () => {
      // registerOldForeignAsset 0
      const { registeredAssetId: registeredAssetIdZero, registeredAsset: registeredAssetZero } =
        await registerOldForeignAsset(context, STATEMINT_LOCATION, assetMetadata);
      assetIdZero = registeredAssetIdZero;
      // registerOldForeignAsset 1
      const { registeredAssetId: registeredAssetIdOne, registeredAsset: registeredAssetOne } =
        await registerOldForeignAsset(context, STATEMINT_ASSET_ONE_LOCATION, assetMetadata, 0, 1);
      assetIdOne = registeredAssetIdOne;

      expect(registeredAssetZero.owner.toHex()).to.eq(palletId.toLowerCase());
      expect(registeredAssetOne.owner.toHex()).to.eq(palletId.toLowerCase());
    });

    it({
      id: "T01",
      title: "Should receive 10 asset 0 tokens using statemint asset 1 as fee",
      test: async function () {
        // We are going to test that, using one of them as fee payment (assetOne),
        // we can receive the other
        const xcmMessage = new XcmFragment({
          assets: [
            {
              multilocation: {
                parents: 1,
                interior: {
                  X3: [
                    { Parachain: statemint_para_id },
                    { PalletInstance: statemint_assets_pallet_instance },
                    { GeneralIndex: 0n },
                  ],
                },
              },
              fungible: 10000000000000n,
            },
            {
              multilocation: {
                parents: 1,
                interior: {
                  X3: [
                    { Parachain: statemint_para_id },
                    { PalletInstance: statemint_assets_pallet_instance },
                    { GeneralIndex: 1n },
                  ],
                },
              },
              fungible: 10000000000000n,
            },
          ],
          weight_limit: {
            refTime: 40000000000n,
            proofSize: 110000n,
          },
          beneficiary: alith.address,
        })
          .reserve_asset_deposited()
          .clear_origin()
          .buy_execution(1) // buy execution with asset at index 1
          .deposit_asset(2n)
          .as_v5();

        // Send an XCM and create block to execute it
        await injectHrmpMessageAndSeal(context, statemint_para_id, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        // Make sure the state has ALITH's foreign parachain tokens
        const alithAssetZeroBalance = (
          await context.polkadotJs().query.assets.account(assetIdZero, alith.address)
        )
          .unwrap()
          .balance.toBigInt();

        expect(alithAssetZeroBalance).to.eq(10n * FOREIGN_TOKEN);
      },
    });
  },
});

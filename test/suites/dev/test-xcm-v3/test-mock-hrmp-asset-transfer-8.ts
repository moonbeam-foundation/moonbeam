import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";

import { alith } from "@moonwall/util";
import {
  registerForeignAsset,
  XcmFragment,
  RawXcmMessage,
  injectHrmpMessageAndSeal,
} from "../../../helpers/xcm.js";

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

describeSuite({
  id: "D3517",
  title: "Mock XCM - receive horizontal transfer",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let assetId: string;

    beforeAll(async () => {
      // registerForeignAsset
      const { registeredAssetId, registeredAsset } = await registerForeignAsset(
        context,
        STATEMINT_LOCATION,
        assetMetadata
      );
      assetId = registeredAssetId;
      expect(registeredAsset.owner.toHex()).to.eq(palletId.toLowerCase());
    });

    it({
      id: "T01",
      title: "Should not receive 10 asset 0 tokens because fee not supported",
      test: async function () {
        // We are going to test that, using one of them as fee payment (assetOne),
        // we can receive the other
        const xcmMessage = new XcmFragment({
          assets: [
            {
              multilocation: {
                parents: 1,
                interior: { X2: [{ Parachain: statemint_para_id }, { GeneralIndex: 0n }] },
              },
              fungible: 10000000000000n,
            },
          ],
          weight_limit: {
            refTime: 4000000000n,
            proofSize: 80000n,
          } as any,
          beneficiary: alith.address,
        })
          .reserve_asset_deposited()
          .clear_origin()
          .buy_execution()
          .deposit_asset_v3(2n)
          .as_v3();

        // Send an XCM and create block to execute it
        await injectHrmpMessageAndSeal(context, statemint_para_id, {
          type: "StagingXcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        // Make sure the state has ALITH's foreign parachain tokens
        const alithAssetZeroBalance = await context
          .polkadotJs()
          .query.assets.account(assetId, alith.address);

        expect(alithAssetZeroBalance.isNone).to.eq(true);
      },
    });
  },
});

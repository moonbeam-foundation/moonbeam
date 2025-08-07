import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";

import { alith } from "@moonwall/util";
import {
  XcmFragment,
  type RawXcmMessage,
  injectHrmpMessageAndSeal,
  registerForeignAsset,
  foreignAssetBalance,
  addAssetToWeightTrader,
} from "../../../../helpers";

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
  id: "D024014",
  title: "Mock XCM - receive horizontal transfer",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const assetId = 1n;

    beforeAll(async () => {
      await registerForeignAsset(context, assetId, STATEMINT_LOCATION, assetMetadata);

      // Note: Intentionally NOT adding to weight trader to test fee not supported scenario
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
          .deposit_asset(2n)
          .as_v3();

        // Send an XCM and create block to execute it
        await injectHrmpMessageAndSeal(context, statemint_para_id, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        // Make sure the state has no ALITH's foreign parachain tokens (fee not supported)
        const alithAssetZeroBalance = await foreignAssetBalance(
          context,
          assetId,
          alith.address as `0x${string}`
        );

        expect(alithAssetZeroBalance).to.eq(0n);
      },
    });
  },
});

import "@moonbeam-network/api-augment";
import { alith, beforeAll, describeSuite, expect } from "moonwall";

import {
  XcmFragment,
  type RawXcmMessage,
  injectHrmpMessageAndSeal,
  registerForeignAsset,
  foreignAssetBalance,
  addAssetToWeightTrader,
} from "../../../../helpers";

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

describeSuite({
  id: "D024101",
  title: "Mock XCM - receive horizontal transfer",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    const assetId = 1n;

    beforeAll(async () => {
      // Register foreign asset
      await registerForeignAsset(context, assetId, STATEMINT_LOCATION, assetMetadata);

      await addAssetToWeightTrader(STATEMINT_LOCATION, 0n, context);
    });

    it({
      id: "T01",
      title: "Should receive a 10 Statemine tokens to Alith with new prefix",
      test: async function () {
        // We are going to test that, using the prefix after
        // https://github.com/paritytech/cumulus/pull/831
        // we can receive the tokens on the assetId registed with the old prefix

        // New prefix:
        // Parachain(Statemint parachain)
        // PalletInstance(Statemint assets pallet instance)
        // GeneralIndex(assetId being transferred)
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
          ],
          weight_limit: {
            refTime: 40000000000n,
            proofSize: 110000n,
          },
          beneficiary: alith.address,
        })
          .reserve_asset_deposited()
          .clear_origin()
          .buy_execution()
          .deposit_asset()
          .as_v4();

        // Send an XCM and create block to execute it
        await injectHrmpMessageAndSeal(context, statemint_para_id, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        // Make sure the state has ALITH's foreign parachain tokens
        const alith_balance_after = await foreignAssetBalance(
          context,
          assetId,
          alith.address as `0x{string}`
        );

        expect(alith_balance_after).to.eq(10n * FOREIGN_TOKEN);
      },
    });
  },
});

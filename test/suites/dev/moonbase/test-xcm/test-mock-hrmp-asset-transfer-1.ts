import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";

import { BN } from "@polkadot/util";
import { alith } from "@moonwall/util";
import {
  XcmFragment,
  type RawXcmMessage,
  injectHrmpMessageAndSeal,
  XCM_VERSIONS,
  convertXcmFragmentToVersion,
  injectHrmpMessage,
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

const FOREIGN_TOKEN = 10n ** assetMetadata.decimals; // 12 decimals

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
  id: "D024008",
  title: "Mock XCM - receive horizontal transfer",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    const assetId = 1n;

    beforeAll(async () => {
      await registerForeignAsset(context, assetId, STATEMINT_LOCATION, assetMetadata);

      await addAssetToWeightTrader(STATEMINT_LOCATION, 0n, context);
    });

    for (const xcmVersion of XCM_VERSIONS) {
      it({
        id: `T01-XCM-v${xcmVersion}`,
        title: "Should receive a horizontal transfer of 10 FOREIGNs to Alith",
        test: async function () {
          const alith_balance_before = await foreignAssetBalance(
            context,
            assetId,
            alith.address as `0x{string}`
          );

          let xcmMessage = new XcmFragment({
            assets: [
              {
                multilocation: STATEMINT_LOCATION.Xcm,
                fungible: FOREIGN_TOKEN,
              },
            ],
            weight_limit: {
              refTime: 10_000_000_000,
              proofSize: 256 * 1024,
            },
            beneficiary: alith.address,
          })
            .reserve_asset_deposited()
            .clear_origin()
            .buy_execution()
            .deposit_asset();

          // Convert to appropriate XCM version
          xcmMessage = convertXcmFragmentToVersion(xcmMessage, xcmVersion);

          // Send RPC call to inject XCM message
          // You can provide a message, but if you don't a horizontal transfer is the default
          await injectHrmpMessage(context, statemint_para_id, {
            type: "XcmVersionedXcm",
            payload: xcmMessage,
          } as RawXcmMessage);

          // Process the next block
          await context.createBlock();
          // Create a block in which the XCM will be executed
          await context.createBlock();
          // Make sure the state has ALITH's foreign parachain tokens
          const alith_balance_after = await foreignAssetBalance(
            context,
            assetId,
            alith.address as `0x{string}`
          );

          expect(alith_balance_after - alith_balance_before).to.eq(FOREIGN_TOKEN);
        },
      });

      it({
        id: `T02-XCM-v${xcmVersion}`,
        title: "Should NOT receive Statemine tokens to Alith with old prefix",
        test: async function () {
          const alith_balance_before = await foreignAssetBalance(
            context,
            assetId,
            alith.address as `0x{string}`
          );

          // We are going to test that, using the prefix prior to
          // https://github.com/paritytech/cumulus/pull/831
          // we cannot receive the tokens on the assetId registed with the old prefix

          // Old prefix:
          // Parachain(Statemint parachain)
          // GeneralIndex(assetId being transferred)
          let xcmMessage = new XcmFragment({
            assets: [
              {
                multilocation: {
                  parents: 1,
                  interior: { X2: [{ Parachain: statemint_para_id }, { GeneralIndex: 0n }] },
                },
                fungible: FOREIGN_TOKEN,
              },
            ],
            weight_limit: new BN(4000000000),
            beneficiary: alith.address,
          })
            .reserve_asset_deposited()
            .clear_origin()
            .buy_execution()
            .deposit_asset();

          // Convert to appropriate XCM version
          xcmMessage = convertXcmFragmentToVersion(xcmMessage, xcmVersion);

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

          // The message execution failed
          expect(alith_balance_before).to.eq(alith_balance_after);
        },
      });
    }
  },
});

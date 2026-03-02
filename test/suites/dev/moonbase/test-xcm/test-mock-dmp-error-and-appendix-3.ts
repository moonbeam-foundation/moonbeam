import "@moonbeam-network/api-augment";
import { alith, beforeAll, customDevRpcRequest, describeSuite, expect } from "moonwall";
import type { XcmVersionedXcm } from "@polkadot/types/lookup";
import {
  RELAY_SOURCE_LOCATION,
  XcmFragment,
  XCM_VERSIONS,
  relayAssetMetadata,
  convertXcmFragmentToVersion,
  registerForeignAsset,
  foreignAssetBalance,
  addAssetToWeightTrader,
} from "../../../../helpers";

// Twelve decimal places in the moonbase relay chain's token
const RELAY_TOKEN = 10n ** relayAssetMetadata.decimals; // 12 decimals

describeSuite({
  id: "D023904",
  title: "Mock XCM - downward transfer with always triggered appendix",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    const assetId = 1n;

    beforeAll(async () => {
      await registerForeignAsset(context, assetId, RELAY_SOURCE_LOCATION, relayAssetMetadata);

      await addAssetToWeightTrader(RELAY_SOURCE_LOCATION, 0n, context);
    });

    for (const xcmVersion of XCM_VERSIONS) {
      it({
        id: `T01-XCM-v${xcmVersion}`,
        title: `Should make sure Alith receives 10 dot with appendix and without error (XCM v${xcmVersion})`,
        test: async function () {
          const initialDotBalance = await foreignAssetBalance(
            context,
            assetId,
            alith.address as `0x{string}`
          );

          let xcmMessage = new XcmFragment({
            assets: [
              {
                multilocation: {
                  parents: 1,
                  interior: {
                    Here: null,
                  },
                },
                fungible: 10n * RELAY_TOKEN,
              },
            ],
            beneficiary: alith.address,
          })
            .reserve_asset_deposited()
            .buy_execution()
            // Set an appendix to be executed after the XCM message is executed. No matter if errors
            .with(function () {
              return this.set_appendix_with([this.deposit_asset]);
            });

          // Convert to appropriate XCM version
          xcmMessage = convertXcmFragmentToVersion(xcmMessage, xcmVersion);

          const receivedMessage: XcmVersionedXcm = context
            .polkadotJs()
            .createType("XcmVersionedXcm", xcmMessage);

          const totalMessage = [...receivedMessage.toU8a()];
          // Send RPC call to inject XCM message
          await customDevRpcRequest("xcm_injectDownwardMessage", [totalMessage]);

          // Create a block in which the XCM will be executed
          await context.createBlock();
          await context.createBlock();
          // Make sure the state has ALITH's to DOT tokens
          const alith_dot_balance = await foreignAssetBalance(
            context,
            assetId,
            alith.address as `0x{string}`
          );

          expect(
            alith_dot_balance - initialDotBalance,
            "Alith should receive exactly 10 DOT more than initial balance"
          ).to.eq(10n * RELAY_TOKEN);
        },
      });
    }
  },
});

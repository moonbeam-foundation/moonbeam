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
  id: "D023902",
  title: "Mock XCM - downward transfer with non-triggered error handler",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const assetId = 1n;

    beforeAll(async () => {
      await registerForeignAsset(context, assetId, RELAY_SOURCE_LOCATION, relayAssetMetadata);

      await addAssetToWeightTrader(RELAY_SOURCE_LOCATION, 1_000_000_000_000_000_000n, context);
    });

    for (const xcmVersion of XCM_VERSIONS) {
      it({
        id: `T01-XCM-v${xcmVersion}`,
        title: `Should make sure that Alith does not receive 10 dot without error (XCM v${xcmVersion})`,
        test: async function () {
          const aliceBeforeBalance = await foreignAssetBalance(
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
            // BuyExecution does not charge for fees because we registered it for not doing so
            // But since there is no error, and the deposit is on the error handler, the assets
            // will be trapped
            .with(function () {
              return this.set_error_handler_with([this.deposit_asset]);
            })
            .clear_origin();

          // Convert to appropriate XCM version
          xcmMessage = convertXcmFragmentToVersion(xcmMessage, xcmVersion);

          const receivedMessage: XcmVersionedXcm = context
            .polkadotJs()
            .createType("XcmVersionedXcm", xcmMessage) as any;

          const totalMessage = [...receivedMessage.toU8a()];
          // Send RPC call to inject XCM message
          await customDevRpcRequest("xcm_injectDownwardMessage", [totalMessage]);

          // Create a block in which the XCM will be executed
          await context.createBlock();
          await context.createBlock();
          // Make sure ALITH did not reveive anything
          const alithAfterBalance = await foreignAssetBalance(
            context,
            assetId,
            alith.address as `0x{string}`
          );

          expect(alithAfterBalance, "Alith's DOT balance is not empty").to.be.equal(
            aliceBeforeBalance
          );
        },
      });
    }
  },
});

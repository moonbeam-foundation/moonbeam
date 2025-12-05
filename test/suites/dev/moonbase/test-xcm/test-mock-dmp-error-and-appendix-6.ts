import "@moonbeam-network/api-augment";
import { beforeAll, customDevRpcRequest, describeSuite, expect } from "@moonwall/cli";
import { alith } from "@moonwall/util";
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
  id: "D023907",
  title: "Mock XCM - downward transfer claim trapped assets",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const assetId = 1n;

    beforeAll(async () => {
      await registerForeignAsset(context, assetId, RELAY_SOURCE_LOCATION, relayAssetMetadata);

      await addAssetToWeightTrader(RELAY_SOURCE_LOCATION, 0n, context);

      // Trap assets for each XCM version
      for (const xcmVersion of XCM_VERSIONS) {
        // BuyExecution does not charge for fees because we registered it for not doing so
        // But since there is no error, and the deposit is on the error handler, the assets
        // will be trapped.
        // Goal is to trap assets, so that later can be claimed
        // Since we only BuyExecution, but we do not do anything with the assets after that,
        // they are trapped
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
        })
          .reserve_asset_deposited()
          .buy_execution();

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
      }

      // Make sure ALITH did not receive anything
      const alith_dot_balance = await foreignAssetBalance(
        context,
        assetId,
        alith.address as `0x{string}`
      );

      expect(alith_dot_balance).to.eq(0n);
    });

    for (const xcmVersion of XCM_VERSIONS) {
      it({
        id: `T01-XCM-v${xcmVersion}`,
        title: `Should make sure that Alith receives claimed assets (XCM v${xcmVersion})`,
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
            // Claim assets that were previously trapped
            // assets: the assets that were trapped
            // ticket: the version of the assets (xcm version)
            .claim_asset()
            .buy_execution()
            // Deposit assets, this time correctly, on Alith
            .deposit_asset();

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
            "Alith's DOT balance should increase by exactly 10 DOT"
          ).to.eq(10n * RELAY_TOKEN);
        },
      });
    }
  },
});

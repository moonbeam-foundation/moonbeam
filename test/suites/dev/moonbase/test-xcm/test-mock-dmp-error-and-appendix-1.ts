import "@moonbeam-network/api-augment";
import { beforeAll, customDevRpcRequest, describeSuite, expect } from "@moonwall/cli";
import { alith } from "@moonwall/util";
import type { XcmVersionedXcm } from "@polkadot/types/lookup";
import {
  RELAY_SOURCE_LOCATION,
  XcmFragment,
  XCM_VERSIONS,
  registerOldForeignAsset,
  relayAssetMetadata,
} from "../../../../helpers";

// Twelve decimal places in the moonbase relay chain's token
const RELAY_TOKEN = 1_000_000_000_000n;

const palletId = "0x6D6f646c617373746d6E67720000000000000000";

describeSuite({
  id: "D024002",
  title: "Mock XCM - downward transfer with non-triggered error handler",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let assetId: string;

    beforeAll(async () => {
      // registerOldForeignAsset
      const { registeredAssetId, registeredAsset } = await registerOldForeignAsset(
        context,
        RELAY_SOURCE_LOCATION,
        relayAssetMetadata
      );
      assetId = registeredAssetId;
      expect(registeredAsset.owner.toHex()).to.eq(palletId.toLowerCase());
    });

    for (const xcmVersion of XCM_VERSIONS) {
      it({
        id: `T01-XCM-v${xcmVersion}`,
        title: `Should make sure that Alith does not receive 10 dot without error (XCM v${xcmVersion})`,
        test: async function () {
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
          if (xcmVersion === 3) {
            xcmMessage = xcmMessage.as_v3();
          } else if (xcmVersion === 4) {
            xcmMessage = xcmMessage.as_v4();
          } else if (xcmVersion === 5) {
            xcmMessage = xcmMessage.as_v5();
          }

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
          const alith_dot_balance = await context
            .polkadotJs()
            .query.assets.account(assetId, alith.address);

          expect(alith_dot_balance.isNone, "Alith's DOT balance is not empty").to.be.true;
        },
      });
    }
  },
});

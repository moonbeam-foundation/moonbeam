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
  convertXcmFragmentToVersion,
} from "../../../../helpers";

// Twelve decimal places in the moonbase relay chain's token
const RELAY_TOKEN = 1_000_000_000_000n;

const palletId = "0x6D6f646c617373746d6E67720000000000000000";

describeSuite({
  id: "D024006",
  title: "Mock XCM - downward transfer with always triggered appendix",
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
        title: `Should make sure Alith receives 10 dot with appendix and error (XCM v${xcmVersion})`,
        test: async function () {
          // Get initial balance
          const initialBalance = await context
            .polkadotJs()
            .query.assets.account(assetId, alith.address);
          const initialDotBalance = initialBalance.isSome
            ? initialBalance.unwrap().balance.toBigInt()
            : 0n;

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
            // As a consequence the trapped assets will be entirely credited
            // The goal is to show appendix runs even if there is an error
            .with(function () {
              return this.set_appendix_with([this.deposit_asset]);
            })
            .trap();

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
          const alith_dot_balance = (
            await context.polkadotJs().query.assets.account(assetId, alith.address)
          )
            .unwrap()
            .balance.toBigInt();

          expect(
            alith_dot_balance - initialDotBalance,
            "Alith should receive exactly 10 DOT more than initial balance"
          ).to.eq(10n * RELAY_TOKEN);
        },
      });
    }
  },
});

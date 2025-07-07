import "@moonbeam-network/api-augment";
import { beforeAll, customDevRpcRequest, describeSuite, expect } from "@moonwall/cli";
import { alith } from "@moonwall/util";
import { BN } from "@polkadot/util";
import {
  PARA_2000_SOURCE_LOCATION,
  registerOldForeignAsset,
  XcmFragment,
  XCM_VERSIONS,
  type RawXcmMessage,
  injectHrmpMessageAndSeal,
  convertXcmFragmentToVersion,
} from "../../../../helpers";

const FOREIGN_TOKEN = 1_000_000_000_000n;

const palletId = "0x6D6f646c617373746d6E67720000000000000000";
const foreign_para_id = 2000;
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
  id: "D024008",
  title: "Mock XCM - receive horizontal transfer",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    // XCM v3 uses a different test scenario than v4/v5
    describe("XCM V3", () => {
      let assetId: string;

      beforeAll(async () => {
        // registerOldForeignAsset
        const { registeredAssetId, registeredAsset } = await registerOldForeignAsset(
          context,
          PARA_2000_SOURCE_LOCATION,
          assetMetadata
        );
        assetId = registeredAssetId;
        expect(registeredAsset.owner.toHex()).to.eq(palletId.toLowerCase());
      });

      it({
        id: "T01-XCM-v3",
        title: "Should receive a horizontal transfer of 10 FOREIGNs to Alith",
        test: async function () {
          // Send RPC call to inject XCM message
          // You can provide a message, but if you don't a horizontal transfer is the default
          await customDevRpcRequest("xcm_injectHrmpMessage", [foreign_para_id, []]);

          // Create a block in which the XCM will be executed
          await context.createBlock();
          await context.createBlock();
          // Make sure the state has ALITH's foreign parachain tokens
          const alith_dot_balance = (
            await context.polkadotJs().query.assets.account(assetId, alith.address)
          )
            .unwrap()
            .balance.toBigInt();

          expect(alith_dot_balance).to.eq(10n * FOREIGN_TOKEN);
        },
      });
    });

    // XCM v4 and v5 use a different test scenario
    describe("XCM V4/V5", () => {
      let assetId: string;

      beforeAll(async () => {
        // registerOldForeignAsset
        const { registeredAssetId, registeredAsset } = await registerOldForeignAsset(
          context,
          STATEMINT_LOCATION,
          assetMetadata
        );
        assetId = registeredAssetId;
        expect(registeredAsset.owner.toHex()).to.eq(palletId.toLowerCase());
      });

      for (const xcmVersion of [4, 5] as const) {
        it({
          id: `T01-XCM-v${xcmVersion}`,
          title: `Should NOT receive a 10 Statemine tokens to Alith with old prefix (XCM v${xcmVersion})`,
          test: async function () {
            // We are going to test that, using the prefix prior to
            // https://github.com/paritytech/cumulus/pull/831
            // we cannot receive the tokens on the assetId registered with the old prefix

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
                  fungible: 10000000000000n,
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
            const alith_dot_balance = await context
              .polkadotJs()
              .query.assets.account(assetId, alith.address);

            // The message execution failed
            expect(alith_dot_balance.isNone).to.be.true;
          },
        });
      }
    });
  },
});

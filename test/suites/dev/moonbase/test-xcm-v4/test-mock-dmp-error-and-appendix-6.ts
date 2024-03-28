import "@moonbeam-network/api-augment";
import { beforeAll, customDevRpcRequest, describeSuite, expect } from "@moonwall/cli";
import { alith } from "@moonwall/util";
import type { XcmVersionedXcm } from "@polkadot/types/lookup";
import {
  RELAY_SOURCE_LOCATION,
  XcmFragment,
  registerForeignAsset,
  relayAssetMetadata,
} from "../../../../helpers";

// Twelve decimal places in the moonbase relay chain's token
const RELAY_TOKEN = 1_000_000_000_000n;

const palletId = "0x6D6f646c617373746d6E67720000000000000000";

describeSuite({
  id: "D014106",
  title: "Mock XCM V3 - downward transfer claim trapped assets",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let assetId: string;

    beforeAll(async () => {
      // registerForeignAsset
      const { registeredAssetId, registeredAsset } = await registerForeignAsset(
        context,
        RELAY_SOURCE_LOCATION,
        relayAssetMetadata
      );
      assetId = registeredAssetId;
      expect(registeredAsset.owner.toHex()).to.eq(palletId.toLowerCase());

      // BuyExecution does not charge for fees because we registered it for not doing so
      // But since there is no error, and the deposit is on the error handler, the assets
      // will be trapped.
      // Goal is to trap assets, so that later can be claimed
      // Since we only BuyExecution, but we do not do anything with the assets after that,
      // they are trapped
      const xcmMessage = new XcmFragment({
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
        .buy_execution()
        .as_v4();

      const receivedMessage: XcmVersionedXcm = context
        .polkadotJs()
        .createType("XcmVersionedXcm", xcmMessage);

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

      expect(alith_dot_balance.isNone).to.be.true;
    });

    it({
      id: "T01",
      title: "Should make sure that Alith receives claimed assets",
      test: async function () {
        const xcmMessage = new XcmFragment({
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
          .deposit_asset_v3()
          .as_v4();

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

        expect(alith_dot_balance, "Alith's DOT balance is empty").to.eq(10n * RELAY_TOKEN);
      },
    });
  },
});

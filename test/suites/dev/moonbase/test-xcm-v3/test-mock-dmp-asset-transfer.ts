import "@moonbeam-network/api-augment";
import { beforeAll, customDevRpcRequest, describeSuite, expect } from "@moonwall/cli";
import { alith } from "@moonwall/util";
import {
  RELAY_SOURCE_LOCATION,
  registerForeignAsset,
  relayAssetMetadata,
} from "../../../../helpers";

// Twelve decimal places in the moonbase relay chain's token
const RELAY_TOKEN = 1_000_000_000_000n;

const palletId = "0x6D6f646c617373746d6E67720000000000000000";

describeSuite({
  id: "D014101",
  title: "Mock XCM - receive downward transfer",
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
    });

    it({
      id: "T01",
      title: "Should receive a downward transfer of 10 DOTs to Alith",
      test: async function () {
        // Send RPC call to inject XCM message
        // You can provide a message, but if you don't a downward transfer is the default
        await customDevRpcRequest("xcm_injectDownwardMessage", [[]]);

        // Process the next block
        await context.createBlock();
        // Create a block in which the XCM will be executed
        await context.createBlock();

        // Make sure the state has ALITH's to DOT tokens
        const alith_dot_balance = (
          await context.polkadotJs().query.assets.account(assetId, alith.address)
        )
          .unwrap()
          .balance.toBigInt();

        expect(alith_dot_balance).to.eq(10n * RELAY_TOKEN);
      },
    });
  },
});

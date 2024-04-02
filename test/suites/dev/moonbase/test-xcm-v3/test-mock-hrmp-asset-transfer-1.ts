import "@moonbeam-network/api-augment";
import { beforeAll, customDevRpcRequest, describeSuite, expect } from "@moonwall/cli";
import { alith } from "@moonwall/util";
import { PARA_2000_SOURCE_LOCATION, registerForeignAsset } from "../../../../helpers";

const FOREIGN_TOKEN = 1_000_000_000_000n;

const palletId = "0x6D6f646c617373746d6E67720000000000000000";
const foreign_para_id = 2000;

const assetMetadata = {
  name: "FOREIGN",
  symbol: "FOREIGN",
  decimals: 12n,
  isFrozen: false,
};

describeSuite({
  id: "D014009",
  title: "Mock XCM - receive horizontal transfer",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let assetId: string;

    beforeAll(async () => {
      // registerForeignAsset
      const { registeredAssetId, registeredAsset } = await registerForeignAsset(
        context,
        PARA_2000_SOURCE_LOCATION,
        assetMetadata
      );
      assetId = registeredAssetId;
      expect(registeredAsset.owner.toHex()).to.eq(palletId.toLowerCase());
    });

    it({
      id: "T01",
      title: "Should receive a horizontal transfer of 10 FOREIGNs to Alith",
      test: async function () {
        // Send RPC call to inject XCM message
        // You can provide a message, but if you don't a horizontal transfer is the default
        await customDevRpcRequest("xcm_injectHrmpMessage", [foreign_para_id, []]);

        // Process the next block
        await context.createBlock();
        // Create a block in which the XCM will be executed
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
  },
});

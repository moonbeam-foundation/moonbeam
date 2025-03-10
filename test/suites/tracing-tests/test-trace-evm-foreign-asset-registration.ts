import { customDevRpcRequest, describeSuite, expect } from "@moonwall/cli";
import {
  ARBITRARY_ASSET_ID,
  registerForeignAsset,
  RELAY_SOURCE_LOCATION_V4,
  relayAssetMetadata,
} from "../../helpers";

describeSuite({
  id: "T17",
  title: "Trace ERC20 Foreign asset creation",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    it({
      id: "T01",
      title: "Ensure native ERC20 foreign asset creation is traceable",
      timeout: 50000,
      test: async function () {
        const assetLocation = RELAY_SOURCE_LOCATION_V4;
        const assetId = ARBITRARY_ASSET_ID;

        // Register the asset
        await registerForeignAsset(context, assetId, assetLocation, relayAssetMetadata);

        const number = await context.viem().getBlockNumber();
        const traces = await customDevRpcRequest("debug_traceBlockByNumber", [
          number.toString(),
          { tracer: "callTracer" },
        ]);

        expect(traces).to.toMatchObject([
          {
            result: {
              to: "0xffffffff1fcacbd218edc0eba20fc2308c778080",
              type: "CREATE",
            },
          },
        ]);
      },
    });
  },
});

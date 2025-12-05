import "@moonbeam-network/api-augment";
import { beforeAll, customDevRpcRequest, describeSuite, expect } from "@moonwall/cli";
import type { ApiPromise } from "@polkadot/api";

describeSuite({
  id: "LL-MOONBEAM-REGRESSIONS",
  title: "Lazy Loading - Regression tests for Moonbeam",
  foundationMethods: "dev",
  testCases: ({ it, context }) => {
    let api: ApiPromise;

    beforeAll(async () => {
      api = context.polkadotJs();
    });

    it({
      id: "T01",
      title: "Validate block containing a EIP7702 transaction",
      test: async function () {
        // Fetch the block containing the EIP7702 transaction (Block 12962274)
        const block = await customDevRpcRequest("eth_getBlockByNumber", ["0xc5c9e2", true]);
        await expect(block).toMatchFileSnapshot("snapshots/moonbeam-block-12962274.json");
      },
    });
  },
});

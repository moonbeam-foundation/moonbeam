import "@moonbeam-network/api-augment";
import { customDevRpcRequest, describeSuite, expect } from "@moonwall/cli";

describeSuite({
  id: "D1702",
  title: "Filter Pending Transaction API",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should not be supported",
      // Looks like this is now supported 🎉
      modifier: "skip",
      test: async function () {
        expect(
          async () => await customDevRpcRequest("eth_newPendingTransactionFilter", [])
        ).rejects.toThrowError("Method not available.");
      },
    });
  },
});

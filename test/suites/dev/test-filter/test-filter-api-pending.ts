import "@moonbeam-network/api-augment";
import { describeSuite, expect, customDevRpcRequest } from "@moonwall/cli";

describeSuite({
  id: "D1702",
  title: "Filter Pending Transaction API",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should not be supported",
      test: async function () {
        expect(
          async () => await customDevRpcRequest("eth_newPendingTransactionFilter", [])
        ).rejects.toThrowError("Method not available.");
      },
    });
  },
});

import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { customDevRpcRequest } from "../../../helpers/common.js";

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

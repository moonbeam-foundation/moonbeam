import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { customDevRpcRequest } from "../../../helpers/common.js";

describeSuite({
  id: "D1202",
  title: "Deprecated RPC",
  foundationMethods: "dev",
  testCases: ({ it }) => {
    const deprecatedMethods = [
      { method: "eth_getCompilers", params: [] },
      { method: "eth_compileLLL", params: ["(returnlll (suicide (caller)))"] },
      {
        method: "eth_compileSolidity",
        params: ["contract test { function multiply(uint a) returns(uint d) {return a * 7;}}"],
      },
      { method: "eth_compileSerpent", params: ["/* some serpent 🐍🐍🐍 */"] },
    ];

    for (const { method, params } of deprecatedMethods) {
      it({
        id: `T0${deprecatedMethods.findIndex((item) => item.method == method) + 1}`,
        title: `${method} should be mark as not found`,
        test: async function () {
          expect(async () => await customDevRpcRequest(method, params)).rejects.toThrowError(
            "Method not found"
          );
        },
      });
    }
  },
});

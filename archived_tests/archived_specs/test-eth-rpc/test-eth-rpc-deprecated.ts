import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { customWeb3Request } from "../../util/providers";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";

describeDevMoonbeam("Deprecated RPC", (context) => {
  // List of deprecated methods
  [
    { method: "eth_getCompilers", params: [] },
    { method: "eth_compileLLL", params: ["(returnlll (suicide (caller)))"] },
    {
      method: "eth_compileSolidity",
      params: ["contract test { function multiply(uint a) returns(uint d) {return a * 7;}}"],
    },
    { method: "eth_compileSerpent", params: ["/* some serpent */"] },
  ].forEach(({ method, params }) => {
    it(`${method} should be mark as not found`, async function () {
      expect(await customWeb3Request(context.web3, method, params)).to.deep.equal({
        id: 1,
        jsonrpc: "2.0",
        error: { message: `Method not found`, code: -32601 },
      });
    });
  });
});

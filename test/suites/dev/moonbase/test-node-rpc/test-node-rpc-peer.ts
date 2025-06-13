import "@moonbeam-network/api-augment";
import { customDevRpcRequest, describeSuite, expect } from "@moonwall/cli";

describeSuite({
  id: "D022301",
  title: "Node - RPC",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    it({
      id: "T01",
      title: "should report peer count in hex",
      test: async function () {
        // this tests that the "net_peerCount" response comes back in hex and not decimal.
        // this seems a bit inconsistent amongst Ethereum APIs, but hex seems to be most common.

        // related: frontier commits 677548c and 78fb3bc
        const result = await customDevRpcRequest("net_peerCount", []);

        // TODO: this is really just testing that the result comes back as a string, not that it's
        //       expressed in hex (as opposed to decimal)
        expect(result).to.be.equal("0x0");
        expect(typeof result).to.be.equal("string");
      },
    });
  },
});

import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { customWeb3Request } from "../../util/providers";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";

describeDevMoonbeam("Node - RPC", (context) => {
  it("should report peer count in hex", async function () {
    // this tests that the "net_peerCount" response comes back in hex and not decimal.
    // this seems a bit inconsistent amongst Ethereum APIs, but hex seems to be most common.

    // related: frontier commits 677548c and 78fb3bc

    const result = await customWeb3Request(context.web3, "net_peerCount", []);

    // TODO: this is really just testing that the result comes back as a string, not that it's
    //       expressed in hex (as opposed to decimal)
    expect(result.result).to.be.equal("0x0");
    expect(typeof result.result).to.be.equal("string");
  });
});

import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { customWeb3Request } from "../../../util/providers";
import { describeDevMoonbeam } from "../../../util/setup-dev-tests";

describeDevMoonbeam("Filter Pending Transaction API", (context) => {
  it("should be supported", async function () {
    // This filter is now supported
    const filter = await customWeb3Request(context.web3, "eth_newPendingTransactionFilter", []);
    expect(filter.result).to.be.eq("0x1");
  });
});

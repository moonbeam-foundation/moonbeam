import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { alith } from "../../util/accounts";
import { getCompiled } from "../../util/contracts";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";

describeDevMoonbeam("Estimate Gas - Contract creation", (context) => {
  it("should return contract creation gas cost", async function () {
    const contract = await getCompiled("MultiplyBy7");
    expect(
      await context.web3.eth.estimateGas({
        from: alith.address,
        data: contract.byteCode,
      })
    ).to.equal(152654);
  });
});

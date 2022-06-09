import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";

import { getCompiled } from "../../util/contracts";
import { alith } from "../../util/accounts";

describeDevMoonbeam("Estimate Gas - Contract creation", (context) => {
  it("should return contract creation gas cost", async function () {
    const contract = await getCompiled("TestContract");
    expect(
      await context.web3.eth.estimateGas({
        from: alith.address,
        data: contract.byteCode,
      })
    ).to.equal(150926);
  });
});

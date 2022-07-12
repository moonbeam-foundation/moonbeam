import "@moonbeam-network/api-augment";

import { expect, use as chaiUse } from "chai";
import chaiAsPromised from "chai-as-promised";

import { describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import { createContract } from "../../util/transactions";

chaiUse(chaiAsPromised);

describeDevMoonbeamAllEthTxTypes("Estimate Gas - infinite loop", (context) => {
  it("Should be able to estimate gas of infinite loop call", async function () {
    const { contract, rawTx } = await createContract(context, "Looper");
    await context.createBlock(rawTx);

    await expect(
      contract.methods.infinite().estimateGas({
        gas: null,
      })
    ).to.be.rejectedWith("gas required exceeds allowance 1500000");
  });
});

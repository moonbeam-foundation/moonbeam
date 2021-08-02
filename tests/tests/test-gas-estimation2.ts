import { expect } from "chai";

import { describeDevMoonbeam } from "../util/setup-dev-tests";

import { createContract } from "../util/transactions";

describeDevMoonbeam("Estimate Gas - infinite loop", (context) => {
  it("Should be able to estimate gas of infinite loop call", async function () {
    const { contract, rawTx } = await createContract(context.web3, "InfiniteContract");
    await context.createBlock({ transactions: [rawTx] });

    expect(
      await contract.methods.infinite().estimateGas({
        gas: null,
      })
    ).to.equal(1_000_000_000);
  });
});

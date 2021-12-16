import { expect } from "chai";

import { createContract } from "../../util/transactions";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { customWeb3Request } from "../../util/providers";

describeDevMoonbeam("TxPool - Genesis", (context) => {
  it("should be empty", async function () {
    let inspect = await customWeb3Request(context.web3, "txpool_inspect", []);
    expect(inspect.result.pending).to.be.empty;
    let content = await customWeb3Request(context.web3, "txpool_content", []);
    expect(content.result.pending).to.be.empty;
  });
});

describeDevMoonbeam("TxPool - New block", (context) => {
  before("Setup: Create transaction and empty block", async () => {
    const { rawTx } = await createContract(context.web3, "TestContract", {
      gas: 1048576,
    });
    await context.createBlock({ transactions: [rawTx] });
    await context.createBlock();
  });

  it("should reset the txpool", async function () {
    let inspect = await customWeb3Request(context.web3, "txpool_inspect", []);
    expect(inspect.result.pending).to.be.empty;
    let content = await customWeb3Request(context.web3, "txpool_content", []);
    expect(content.result.pending).to.be.empty;
  });
});

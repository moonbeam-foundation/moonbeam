import { expect } from "chai";

import { baltathar } from "../../util/accounts";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createTransfer } from "../../util/transactions";

describeDevMoonbeam("Transaction Index", (context) => {
  before("Setup: Create block with transaction", async () => {
    await context.createBlock(createTransfer(context, baltathar.address, 0));
  });
  it("should get transaction by index", async function () {
    const block = 1;
    const index = 0;
    let result = await context.web3.eth.getTransactionFromBlock(block, index);
    expect(result.transactionIndex).to.equal(index);
  });
  it("should return out of bounds message", async function () {
    const block = 0;
    const index = 0;
    await context.web3.eth
      .getTransactionFromBlock(block, index)
      .then((err) => {
        throw new Error(`Not expected to succeed`);
      })
      .catch((err) => expect(err.message).to.equal(`Returned error: ${index} is out of bounds`));
  });
});

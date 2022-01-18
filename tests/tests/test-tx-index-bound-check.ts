import { expect } from "chai";
import { createTransfer } from "../util/transactions";
import { describeDevMoonbeam } from "../util/setup-dev-tests";

describeDevMoonbeam("Transaction Index", (context) => {
  before("Setup: Create block with transaction", async () => {
    const testAccount = "0x1111111111111111111111111111111111111111";
    await context.createBlock({
      transactions: [await createTransfer(context, testAccount, 0)],
    });
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

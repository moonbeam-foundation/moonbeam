import { expect } from "chai";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createContract } from "../util/transactions";
import { GENESIS_ACCOUNT } from "../util/constants";

describeDevMoonbeam("Event - Contract", (context) => {
  it("should contain event", async function () {
    const { rawTx } = await createContract(context, "SingleEventContract", {
      from: GENESIS_ACCOUNT,
    });
    const { txResults } = await context.createBlock({ transactions: [rawTx] });
    const receipt = await context.web3.eth.getTransactionReceipt(txResults[0].result);

    expect(receipt.logs.length).to.be.eq(1);
    expect(
      "0x" + receipt.logs[0].topics[1].substring(26, receipt.logs[0].topics[1].length + 1)
    ).to.be.eq(GENESIS_ACCOUNT.toLowerCase()); // web3 doesn't checksum
  });
});

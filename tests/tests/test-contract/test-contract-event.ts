import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { alith } from "../../util/accounts";
import { describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import { createContract } from "../../util/transactions";

describeDevMoonbeamAllEthTxTypes("Contract - Event", (context) => {
  it("should contain event", async function () {
    const { rawTx } = await createContract(context, "EventEmitter", { from: alith.address });
    const { result } = await context.createBlock(rawTx);
    const receipt = await context.web3.eth.getTransactionReceipt(result.hash);

    expect(receipt.logs.length).to.be.eq(1);
    expect(
      "0x" + receipt.logs[0].topics[1].substring(26, receipt.logs[0].topics[1].length + 1)
    ).to.be.eq(alith.address.toLowerCase()); // web3 doesn't checksum
  });
});

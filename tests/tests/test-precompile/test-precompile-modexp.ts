import { expect } from "chai";
import { describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import { createContract } from "../../util/transactions";

describeDevMoonbeamAllEthTxTypes("Precompiles - ModExp", (context) => {
  it("should be accessible from a smart contract", async function () {
    // See also the ModExp unit tests at
    // github.com/paritytech/frontier/blob/378221a4/frame/evm/precompile/modexp/src/lib.rs#L101
    const { contract, rawTx } = await createContract(context, "ModularCheck");
    const { txResults } = await context.createBlock({ transactions: [rawTx] });

    // The contract should deploy successfully and the receipt should show success.
    const receipt = await context.web3.eth.getTransactionReceipt(txResults[0].result);
    expect(receipt.status).to.be.true;
  });
});

import { expect } from "chai";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createContract } from "../util/transactions";

describeDevMoonbeam("Precompiles - Blake2", (context) => {
  it("should be accessible from a smart contract", async function () {
    const { contract, rawTx } = await createContract(context.web3, "Blake2Check");
    const { txResults } = await context.createBlock({ transactions: [rawTx] });

    // The contract should deploy successfully and the receipt should show success.
    const receipt = await context.web3.eth.getTransactionReceipt(txResults[0].result);
    expect(receipt.status).to.be.true;

    // invoke the contract's test function 'callF'
    const result = await contract.methods.callF().call();
    console.log("RESULTS: ", result);
    // expect(result[0]).to.equal("0xba80a53f981c4d0d6a2797b69f12f6e94c212f14685ac4b74b12bb6fdbffa2d1");
    // expect(result[1]).to.equal("0x7d87c5392aab792dc252d5de4533cc9518d38aa8dbf1925ab92386edd4009923");
    expect(result).to.have.members(
      [
        "0xba80a53f981c4d0d6a2797b69f12f6e94c212f14685ac4b74b12bb6fdbffa2d1",
        "0x7d87c5392aab792dc252d5de4533cc9518d38aa8dbf1925ab92386edd4009923",
      ]
    );
  });
});

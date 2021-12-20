import { expect } from "chai";
import { verifyLatestBlockFees } from "../../util/block";
import { describeDevMoonbeam, describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import { createContract } from "../../util/transactions";
import { customWeb3Request } from "../../util/providers";

describeDevMoonbeamAllEthTxTypes("Contract creation", (context) => {
  it("should return the transaction hash", async () => {
    const { rawTx } = await createContract(context, "TestContract");
    const { txResults } = await context.createBlock({ transactions: [rawTx] });

    expect(
      txResults[0].result,
      "0x286fc7f456a452abb22bc37974fe281164e53ce6381583c8febaa89c92f31c0b"
    );
  });
});

describeDevMoonbeamAllEthTxTypes("Contract creation", (context) => {
  it("should not contain contract at genesis", async function () {
    const { contract } = await createContract(context, "TestContract");
    expect(await context.web3.eth.getCode(contract.options.address)).to.deep.equal("0x");
  });

  it("should store the code on chain", async function () {
    const code =
      "0x608060405234801561001057600080fd5b506004361061002b5760003560e01c8063c6888fa114610030575b" +
      "600080fd5b61004a6004803603810190610045919061008b565b610060565b60405161005791906100c3565b" +
      "60405180910390f35b600060078261006f91906100de565b9050919050565b60008135905061008581610171" +
      "565b92915050565b60006020828403121561009d57600080fd5b60006100ab84828501610076565b91505092" +
      "915050565b6100bd81610138565b82525050565b60006020820190506100d860008301846100b4565b929150" +
      "50565b60006100e982610138565b91506100f483610138565b9250817fffffffffffffffffffffffffffffff" +
      "ffffffffffffffffffffffffffffffffff048311821515161561012d5761012c610142565b5b828202905092" +
      "915050565b6000819050919050565b7f4e487b71000000000000000000000000000000000000000000000000" +
      "00000000600052601160045260246000fd5b61017a81610138565b811461018557600080fd5b5056fea26469" +
      "70667358221220a82dff050f5e40b874671c1f40e579b5a8c361f5313d1a9d32437222ab6a384c64736f6c63" +
      "430008030033";
    const { contract, rawTx } = await createContract(context, "TestContract");
    await customWeb3Request(context.web3, "eth_sendRawTransaction", [rawTx]);
    expect(await context.web3.eth.getCode(contract.options.address, "pending")).to.deep.equal(code);
    await context.createBlock();
    expect(await context.web3.eth.getCode(contract.options.address)).to.deep.equal(code);
  });
});

describeDevMoonbeamAllEthTxTypes("Contract creation -block fees", (context) => {
  it("should check latest block fees", async function () {
    const { rawTx } = await createContract(context, "TestContract");
    const {} = await context.createBlock({ transactions: [rawTx] });
    await verifyLatestBlockFees(context, expect);
  });
});

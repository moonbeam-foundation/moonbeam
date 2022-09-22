import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { verifyLatestBlockFees } from "../../util/block";
import { getCompiled } from "../../util/contracts";
import { customWeb3Request } from "../../util/providers";
import { describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import { createContract } from "../../util/transactions";

describeDevMoonbeamAllEthTxTypes("Contract creation", (context) => {
  it("should return the transaction hash", async () => {
    const { rawTx } = await createContract(context, "MultiplyBy7");
    const { result } = await context.createBlock(rawTx);

    expect(result.hash, "0x286fc7f456a452abb22bc37974fe281164e53ce6381583c8febaa89c92f31c0b");
  });
});

describeDevMoonbeamAllEthTxTypes("eth_call contract create", (context) => {
  it("should return the contract code", async () => {
    const contractData = getCompiled("MultiplyBy7");
    let callCode = await context.web3.eth.call({ data: contractData.byteCode });
    const { rawTx } = await createContract(context, "MultiplyBy7");
    const { result } = await context.createBlock(rawTx);
    let receipt = await context.web3.eth.getTransactionReceipt(result.hash);
    let deployedCode = await context.web3.eth.getCode(receipt.contractAddress);
    expect(callCode).to.be.eq(deployedCode);
  });
});

describeDevMoonbeamAllEthTxTypes("Contract creation", (context) => {
  it("should not contain contract at genesis", async function () {
    const { contract } = await createContract(context, "MultiplyBy7");
    expect(await context.web3.eth.getCode(contract.options.address)).to.deep.equal("0x");
  });

  it("should store the code on chain", async function () {
    await context.createBlock();
    const code =
      "0x608060405234801561001057600080fd5b506004361061002b5760003560e01c8063c6888fa114610030575b" +
      "600080fd5b61004a600480360381019061004591906100b1565b610060565b60405161005791906100ed565b60" +
      "405180910390f35b600060078261006f9190610137565b9050919050565b600080fd5b6000819050919050565b" +
      "61008e8161007b565b811461009957600080fd5b50565b6000813590506100ab81610085565b92915050565b60" +
      "00602082840312156100c7576100c6610076565b5b60006100d58482850161009c565b91505092915050565b61" +
      "00e78161007b565b82525050565b600060208201905061010260008301846100de565b92915050565b7f4e487b" +
      "7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b6000" +
      "6101428261007b565b915061014d8361007b565b9250817fffffffffffffffffffffffffffffffffffffffffff" +
      "ffffffffffffffffffffff048311821515161561018657610185610108565b5b82820290509291505056fea264" +
      "6970667358221220d1329a788d122e50b149670ac5aa79a040600a6da2033dec05c685b5798f70b164736f6c63" +
      "4300080b0033";
    const { contract, rawTx } = await createContract(context, "MultiplyBy7");
    await customWeb3Request(context.web3, "eth_sendRawTransaction", [rawTx]);
    expect(await context.web3.eth.getCode(contract.options.address, "pending")).to.deep.equal(code);
    await context.createBlock();
    expect(await context.web3.eth.getCode(contract.options.address)).to.deep.equal(code);
  });
});

describeDevMoonbeamAllEthTxTypes("Contract creation -block fees", (context) => {
  it("should check latest block fees", async function () {
    const { rawTx } = await createContract(context, "MultiplyBy7");
    await context.createBlock(rawTx);
    await verifyLatestBlockFees(context);
  });
});

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
      "0x6080604052348015600f57600080fd5b506004361060285760003560e01c8063c6888fa114602d575b600080" +
      "fd5b603c6038366004605f565b604e565b60405190815260200160405180910390f35b60006059826007607756" +
      "5b92915050565b600060208284031215607057600080fd5b5035919050565b8082028115828204841417605957" +
      "634e487b7160e01b600052601160045260246000fdfea26469706673582212206e801ab9e7014f4100a125beec" +
      "52c5d0ee8a7df2b0793f2f8457ad52823589ba64736f6c63430008130033";
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

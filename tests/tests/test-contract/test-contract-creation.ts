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
      "0x608060405234801561005d5760405162461bcd60e51b815260206004820152602260248201527f4574686572" +
      "2073656e7420746f206e6f6e2d70617961626c652066756e637469604482019081526137b760f11b6064830152" +
      "608482fd5b50600436106100785760003560e01c8063c6888fa1146100dd575b60405162461bcd60e51b815260" +
      "206004820152603560248201527f436f6e747261637420646f6573206e6f7420686176652066616c6c6261636b" +
      "2060448201908152746e6f7220726563656976652066756e6374696f6e7360581b6064830152608482fd5b6100" +
      "f06100eb366004610115565b610102565b60405190815260200160405180910390f35b600061010f8260076101" +
      "79565b92915050565b6000602082840312156101725760405162461bcd60e51b81526020600482015260226024" +
      "8201527f414249206465636f64696e673a207475706c65206461746120746f6f2073686f6044820152611c9d60" +
      "f21b6064820152608481fd5b5035919050565b808202811582820484141761010f57634e487b7160e01b600052" +
      "601160045260246000fdfea26469706673582212201908894ace7c2455a9a9c3f237348fbb18e18147a95c2fd7" +
      "096a971132e2f57f64736f6c63430008130033";
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

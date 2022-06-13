import "@moonbeam-network/api-augment";
import { expect, use as chaiUse } from "chai";
import chaiAsPromised from "chai-as-promised";
import { ethers } from "ethers";

import { describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";

import { TransactionReceipt } from "web3-core";
import { getCompiled } from "../../util/contracts";

import { createContract } from "../../util/transactions";
import { Contract } from "web3-eth-contract";
import { alith } from "../../util/accounts";

chaiUse(chaiAsPromised);

describeDevMoonbeamAllEthTxTypes("Estimate Gas - Multiply", (context) => {
  let multContract: Contract;

  before("Setup: Create simple context", async function () {
    const { contract, rawTx } = await createContract(context, "TestContract");
    await context.createBlock(rawTx);
    multContract = contract;
  });

  it("should return correct gas estimation", async function () {
    expect(await multContract.methods.multiply(3).estimateGas()).to.equal(22405);
  });

  it("should work without gas limit", async function () {
    expect(
      await multContract.methods.multiply(3).estimateGas({
        gas: null,
      })
    ).to.equal(22405);
  });

  it("should work with gas limit", async function () {
    expect(
      await multContract.methods.multiply(3).estimateGas({
        gas: 22405,
      })
    ).to.lessThanOrEqual(22405);
  });

  it("should ignore from balance (?)", async function () {
    expect(
      await multContract.methods.multiply(3).estimateGas({
        from: "0x0000000000000000000000000000000000000000",
      })
    ).to.equal(22405);
  });

  it("should not work with a lower gas limit", async function () {
    await expect(
      multContract.methods.multiply(3).estimateGas({
        gas: 21900,
      })
    ).to.be.rejectedWith("gas required exceeds allowance 21900");
  });
});

describeDevMoonbeamAllEthTxTypes("Estimate Gas - Supplied estimate is sufficient", (context) => {
  it("should estimate sufficient gas for creation", async function () {
    const contract = await getCompiled("Incrementer");
    // ask RPC for an gas estimate of deploying this contract
    const estimate = await context.web3.eth.estimateGas({
      from: alith.address,
      data: contract.byteCode,
    });

    // attempt a transaction with our estimated gas
    const { rawTx } = await createContract(context, "Incrementer", { gas: estimate });
    const { result } = await context.createBlock(rawTx);
    const receipt: TransactionReceipt = await context.web3.eth.getTransactionReceipt(result.hash);

    // the transaction should succeed because the estimate should have been sufficient
    expect(receipt.status).to.equal(true);
  });
});

describeDevMoonbeamAllEthTxTypes("Estimate Gas - Handle Gas price", (context) => {
  it("eth_estimateGas 0x0 gasPrice is equivalent to not setting one", async function () {
    const contract = await getCompiled("Incrementer");
    let result = await context.web3.eth.estimateGas({
      from: alith.address,
      data: contract.byteCode,
      gasPrice: "0x0",
    });
    expect(result).to.equal(152884);
    result = await context.web3.eth.estimateGas({
      from: alith.address,
      data: contract.byteCode,
    });
    expect(result).to.equal(152884);
  });
});

describeDevMoonbeamAllEthTxTypes("Estimate Gas - Batch precompile", (context) => {
  it("all batch functions should estimate the same cost", async function () {
    const { contract: contractProxy, rawTx } = await createContract(context, "TestCallList");
    await context.createBlock(rawTx);
    const { contract: contractDummy, rawTx: rawTx2 } = await createContract(
      context,
      "TestContract"
    );
    await context.createBlock(rawTx2);

    const proxyInterface = new ethers.utils.Interface(
      (await getCompiled("TestCallList")).contract.abi
    );
    const dummyInterface = new ethers.utils.Interface(
      (await getCompiled("TestContract")).contract.abi
    );

    const batchInterface = new ethers.utils.Interface((await getCompiled("Batch")).contract.abi);

    const callParameters = [
      [contractProxy.options.address, contractProxy.options.address],
      [],
      [
        proxyInterface.encodeFunctionData("call", [
          contractDummy.options.address,
          dummyInterface.encodeFunctionData("multiply", [42]),
        ]),
        proxyInterface.encodeFunctionData("delegateCall", [
          contractDummy.options.address,
          dummyInterface.encodeFunctionData("multiply", [42]),
        ]),
      ],
      [],
    ];

    const batchSomeGas = await context.web3.eth.estimateGas({
      from: alith.address,
      to: "0x0000000000000000000000000000000000000808",
      data: batchInterface.encodeFunctionData("batchSome", callParameters),
    });

    const batchSomeUntilFailureGas = await context.web3.eth.estimateGas({
      from: alith.address,
      to: "0x0000000000000000000000000000000000000808",
      data: batchInterface.encodeFunctionData("batchSomeUntilFailure", callParameters),
    });

    const batchAllGas = await context.web3.eth.estimateGas({
      from: alith.address,
      to: "0x0000000000000000000000000000000000000808",
      data: batchInterface.encodeFunctionData("batchAll", callParameters),
    });

    expect(batchSomeGas).to.be.eq(batchAllGas);
    expect(batchSomeUntilFailureGas).to.be.eq(batchAllGas);
  });
});

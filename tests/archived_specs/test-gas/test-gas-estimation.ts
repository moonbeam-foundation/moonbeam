import "@moonbeam-network/api-augment";

import { expect, use as chaiUse } from "chai";
import chaiAsPromised from "chai-as-promised";
import { ethers } from "ethers";
import { TransactionReceipt } from "web3-core";
import { Contract } from "web3-eth-contract";
import { customWeb3Request } from "../../../util/providers";

import { alith, faith } from "../../../util/accounts";
import { getAllContracts, getCompiled } from "../../../util/contracts";
import { expectEVMResult } from "../../../util/eth-transactions";
import { describeDevMoonbeamAllEthTxTypes } from "../../../util/setup-dev-tests";
import { createContract } from "../../../util/transactions";
import { PRECOMPILE_BATCH_ADDRESS } from "../../../util/constants";

chaiUse(chaiAsPromised);

describeDevMoonbeamAllEthTxTypes("Estimate Gas - Multiply", (context) => {
  let multContract: Contract;

  before("Setup: Create simple context", async function () {
    const { contract, rawTx } = await createContract(context, "MultiplyBy7");
    await context.createBlock(rawTx);
    multContract = contract;
  });

  it("should return correct gas estimation", async function () {
    expect(await multContract.methods.multiply(3).estimateGas()).to.equal(22364);
  });

  it("should work without gas limit", async function () {
    expect(
      await multContract.methods.multiply(3).estimateGas({
        gas: null,
      })
    ).to.equal(22364);
  });

  it("should work with gas limit", async function () {
    expect(
      await multContract.methods.multiply(3).estimateGas({
        gas: 22364,
      })
    ).to.lessThanOrEqual(22364);
  });

  it("should ignore from balance (?)", async function () {
    expect(
      await multContract.methods.multiply(3).estimateGas({
        from: "0x0000000000000000000000000000000000000000",
      })
    ).to.equal(22364);
  });

  it("should not work with a lower gas limit", async function () {
    await expect(
      multContract.methods.multiply(3).estimateGas({
        gas: 21000,
      })
    ).to.be.rejectedWith("gas required exceeds allowance 21000");
  });
});

describeDevMoonbeamAllEthTxTypes("Estimate Gas - Contract estimation", (context) => {
  const contractNames = getAllContracts();

  before("Init build block", async function () {
    // Estimation for storage need to happen in a block > than genesis.
    // Otherwise contracts that uses block number as storage will remove instead of storing
    // (as block.number == H256::default).
    await context.createBlock();
  });

  it("should have at least 1 contract to estimate", async function () {
    expect(contractNames).length.to.be.at.least(1);
  });

  for (const contractName of contractNames) {
    it(`should be enough for contract ${contractName}`, async function () {
      const contract = getCompiled(contractName);
      const constructorAbi = contract.contract.abi.find((call) => call.type == "constructor");
      // ask RPC for an gas estimate of deploying this contract

      const web3Contract = new context.web3.eth.Contract(contract.contract.abi);
      const args = constructorAbi
        ? constructorAbi.inputs.map((input) =>
            input.type == "bool"
              ? true
              : input.type == "address"
              ? faith.address
              : input.type == "uint256"
              ? `0x${Buffer.from(ethers.utils.randomBytes(32)).toString("hex")}`
              : "0x"
          )
        : [];

      let estimate: number;
      let creationResult: "Revert" | "Succeed";
      try {
        estimate = await web3Contract
          .deploy({
            arguments: args,
            data: contract.byteCode,
          })
          .estimateGas();
        creationResult = "Succeed";
      } catch (e) {
        if (e == "Error: Returned error: VM Exception while processing transaction: revert") {
          estimate = 12_000_000;
          creationResult = "Revert";
        } else {
          throw e;
        }
      }

      // attempt a transaction with our estimated gas
      const { rawTx } = await createContract(context, contractName, { gas: estimate }, args);
      const { result } = await context.createBlock(rawTx);
      const receipt: TransactionReceipt = await context.web3.eth.getTransactionReceipt(result.hash);

      expectEVMResult(result.events, creationResult);
      expect(receipt.status).to.equal(creationResult == "Succeed");
    });
  }
});

describeDevMoonbeamAllEthTxTypes("Estimate Gas - Contract estimation", (context) => {
  it(`evm should return invalid opcode`, async function () {
    let estimate = await customWeb3Request(context.web3, "eth_estimateGas", [
      {
        from: alith.address,
        data: "0xe4",
      },
    ]);
    expect((estimate.error as any).message).to.equal("evm error: InvalidCode(Opcode(228))");
  });
});

describeDevMoonbeamAllEthTxTypes("Estimate Gas - Handle Gas price", (context) => {
  it("eth_estimateGas 0x0 gasPrice is equivalent to not setting one", async function () {
    const contract = getCompiled("Incrementor");
    let result = await context.web3.eth.estimateGas({
      from: alith.address,
      data: contract.byteCode,
      gasPrice: "0x0",
    });
    expect(result).to.equal(174798);
    result = await context.web3.eth.estimateGas({
      from: alith.address,
      data: contract.byteCode,
    });
    expect(result).to.equal(174798);
  });
});

describeDevMoonbeamAllEthTxTypes("Estimate Gas - Batch precompile", (context) => {
  it("all batch functions should estimate the same cost", async function () {
    const { contract: contractProxy, rawTx } = await createContract(context, "CallForwarder");
    await context.createBlock(rawTx);
    const { contract: contractDummy, rawTx: rawTx2 } = await createContract(context, "MultiplyBy7");
    await context.createBlock(rawTx2);

    const proxyInterface = new ethers.utils.Interface(getCompiled("CallForwarder").contract.abi);
    const dummyInterface = new ethers.utils.Interface(getCompiled("MultiplyBy7").contract.abi);

    const batchInterface = new ethers.utils.Interface(
      getCompiled("precompiles/batch/Batch").contract.abi
    );

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
      to: PRECOMPILE_BATCH_ADDRESS,
      data: batchInterface.encodeFunctionData("batchSome", callParameters),
    });

    const batchSomeUntilFailureGas = await context.web3.eth.estimateGas({
      from: alith.address,
      to: PRECOMPILE_BATCH_ADDRESS,
      data: batchInterface.encodeFunctionData("batchSomeUntilFailure", callParameters),
    });

    const batchAllGas = await context.web3.eth.estimateGas({
      from: alith.address,
      to: PRECOMPILE_BATCH_ADDRESS,
      data: batchInterface.encodeFunctionData("batchAll", callParameters),
    });

    expect(batchSomeGas).to.be.eq(batchAllGas);
    expect(batchSomeUntilFailureGas).to.be.eq(batchAllGas);
  });
});

describeDevMoonbeamAllEthTxTypes("Estimate Gas - EOA", (context) => {
  it("Non-transactional calls allowed from e.g. precompile address", async function () {
    const contract = getCompiled("MultiplyBy7");
    expect(
      await context.web3.eth.estimateGas({
        from: PRECOMPILE_BATCH_ADDRESS,
        data: contract.byteCode,
      })
    ).to.equal(157029);
  });
});

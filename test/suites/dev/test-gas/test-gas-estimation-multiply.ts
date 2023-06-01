import "@moonbeam-network/api-augment";
import { describeSuite, beforeAll, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, deployCreateCompiledContract } from "@moonwall/util";
import { Abi } from "abitype";

describeSuite({
  id: "D1704",
  title: "Estimate Gas - Multiply",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let multiAbi: Abi;
    let multiAddress: `0x${string}`;

    beforeAll(async function () {
      const { abi, contractAddress } = await deployCreateCompiledContract(context, "MultiplyBy7");

      multiAbi = abi;
      multiAddress = contractAddress;
    });

    it({
      id: "T01",
      title: "should return correct gas estimation",
      test: async function () {
        const estimatedGas = await context.viemClient("public").estimateContractGas({
          account: ALITH_ADDRESS,
          abi: multiAbi,
          address: multiAddress,
          functionName: "multiply",
          maxPriorityFeePerGas: 0n,
          args: [3],
          value: 0n,
        });

        expect(estimatedGas).to.equal(22364n);
      },
    });

    it({
      id: "T02",
      title: "should work without gas limit",
      test: async function () {
        const estimatedGas = await context.viemClient("public").estimateContractGas({
          account: ALITH_ADDRESS,
          abi: multiAbi,
          address: multiAddress,
          functionName: "multiply",
          maxPriorityFeePerGas: 0n,
          args: [3],
          //@ts-expect-error
          gasLimit: undefined,
          value: 0n,
        });

        expect(estimatedGas).to.equal(22364n);
      },
    });

    it({
      id: "T03",
      title: "should work with gas limit",
      test: async function () {
        const estimatedGas = await context.viemClient("public").estimateContractGas({
          account: ALITH_ADDRESS,
          abi: multiAbi,
          address: multiAddress,
          functionName: "multiply",
          args: [3],
          //@ts-expect-error
          gasLimit: 22364n,
          value: 0n,
        });

        expect(estimatedGas).to.equal(22364n);
      },
    });

    it({
      id: "T04",
      title: "should ignore from balance (?)",
      test: async function () {
        const estimatedGas = await context.viemClient("public").estimateContractGas({
          account: "0x0000000000000000000000000000000000000000",
          abi: multiAbi,
          address: multiAddress,
          functionName: "multiply",
          maxPriorityFeePerGas: 0n,
          args: [3],
          //@ts-expect-error
          gasLimit: 22364n,
          value: 0n,
        });

        expect(estimatedGas).to.equal(22364n);
      },
    });

    it({
      id: "T05",
      title: "should not work with a lower gas limit",
      test: async function () {
        expect(
          async () =>
            await context.viemClient("public").estimateContractGas({
              account: "0x0000000000000000000000000000000000000000",
              abi: multiAbi,
              address: multiAddress,
              functionName: "multiply",
              maxPriorityFeePerGas: 0n,
              args: [3],
              gas: 21000n,
              value: 0n,
            })
        ).rejects.toThrowError("gas required exceeds allowance 21000");
      },
    });
  },
});


// describeDevMoonbeamAllEthTxTypes("Estimate Gas - Contract estimation", (context) => {
//   const contractNames = getAllContracts();

//   before("Init build block", async function () {
//     // Estimation for storage need to happen in a block > than genesis.
//     // Otherwise contracts that uses block number as storage will remove instead of storing
//     // (as block.number == H256::default).
//     await context.createBlock();
//   });

//   it("should have at least 1 contract to estimate", async function () {
//     expect(contractNames).length.to.be.at.least(1);
//   });

//   for (const contractName of contractNames) {
//     it(`should be enough for contract ${contractName}`, async function () {
//       const contract = getCompiled(contractName);
//       const constructorAbi = contract.contract.abi.find((call) => call.type == "constructor");
//       // ask RPC for an gas estimate of deploying this contract

//       const web3Contract = new context.web3.eth.Contract(contract.contract.abi);
//       const args = constructorAbi
//         ? constructorAbi.inputs.map((input) =>
//             input.type == "bool"
//               ? true
//               : input.type == "address"
//               ? faith.address
//               : input.type == "uint256"
//               ? `0x${Buffer.from(ethers.utils.randomBytes(32)).toString("hex")}`
//               : "0x"
//           )
//         : [];

//       let estimate: number;
//       let creationResult: "Revert" | "Succeed";
//       try {
//         estimate = await web3Contract
//           .deploy({
//             arguments: args,
//             data: contract.byteCode,
//           })
//           .estimateGas();
//         creationResult = "Succeed";
//       } catch (e) {
//         if (e == "Error: Returned error: VM Exception while processing transaction: revert") {
//           estimate = 12_000_000;
//           creationResult = "Revert";
//         } else {
//           throw e;
//         }
//       }

//       // attempt a transaction with our estimated gas
//       const { rawTx } = await createContract(context, contractName, { gas: estimate }, args);
//       const { result } = await context.createBlock(rawTx);
//       const receipt: TransactionReceipt = await context.web3.eth.getTransactionReceipt(result.hash);

//       expectEVMResult(result.events, creationResult);
//       expect(receipt.status).to.equal(creationResult == "Succeed");
//     });
//   }
// });

// describeDevMoonbeamAllEthTxTypes("Estimate Gas - Contract estimation", (context) => {
//   it(`evm should return invalid opcode`, async function () {
//     let estimate = await customWeb3Request(context.web3, "eth_estimateGas", [
//       {
//         from: alith.address,
//         data: "0xe4",
//       },
//     ]);
//     expect((estimate.error as any).message).to.equal("evm error: InvalidCode(Opcode(228))");
//   });
// });

// describeDevMoonbeamAllEthTxTypes("Estimate Gas - Handle Gas price", (context) => {
//   it("eth_estimateGas 0x0 gasPrice is equivalent to not setting one", async function () {
//     const contract = getCompiled("Incrementor");
//     let result = await context.web3.eth.estimateGas({
//       from: alith.address,
//       data: contract.byteCode,
//       gasPrice: "0x0",
//     });
//     expect(result).to.equal(174759);
//     result = await context.web3.eth.estimateGas({
//       from: alith.address,
//       data: contract.byteCode,
//     });
//     expect(result).to.equal(174759);
//   });
// });

// describeDevMoonbeamAllEthTxTypes("Estimate Gas - Batch precompile", (context) => {
//   it("all batch functions should estimate the same cost", async function () {
//     const { contract: contractProxy, rawTx } = await createContract(context, "CallForwarder");
//     await context.createBlock(rawTx);
//     const { contract: contractDummy, rawTx: rawTx2 } = await createContract(context, "MultiplyBy7");
//     await context.createBlock(rawTx2);

//     const proxyInterface = new ethers.utils.Interface(getCompiled("CallForwarder").contract.abi);
//     const dummyInterface = new ethers.utils.Interface(getCompiled("MultiplyBy7").contract.abi);

//     const batchInterface = new ethers.utils.Interface(
//       getCompiled("precompiles/batch/Batch").contract.abi
//     );

//     const callParameters = [
//       [contractProxy.options.address, contractProxy.options.address],
//       [],
//       [
//         proxyInterface.encodeFunctionData("call", [
//           contractDummy.options.address,
//           dummyInterface.encodeFunctionData("multiply", [42]),
//         ]),
//         proxyInterface.encodeFunctionData("delegateCall", [
//           contractDummy.options.address,
//           dummyInterface.encodeFunctionData("multiply", [42]),
//         ]),
//       ],
//       [],
//     ];

//     const batchSomeGas = await context.web3.eth.estimateGas({
//       from: alith.address,
//       to: PRECOMPILE_BATCH_ADDRESS,
//       data: batchInterface.encodeFunctionData("batchSome", callParameters),
//     });

//     const batchSomeUntilFailureGas = await context.web3.eth.estimateGas({
//       from: alith.address,
//       to: PRECOMPILE_BATCH_ADDRESS,
//       data: batchInterface.encodeFunctionData("batchSomeUntilFailure", callParameters),
//     });

//     const batchAllGas = await context.web3.eth.estimateGas({
//       from: alith.address,
//       to: PRECOMPILE_BATCH_ADDRESS,
//       data: batchInterface.encodeFunctionData("batchAll", callParameters),
//     });

//     expect(batchSomeGas).to.be.eq(batchAllGas);
//     expect(batchSomeUntilFailureGas).to.be.eq(batchAllGas);
//   });
// });

// describeDevMoonbeamAllEthTxTypes("Estimate Gas - EOA", (context) => {
//   it("Non-transactional calls allowed from e.g. precompile address", async function () {
//     const contract = getCompiled("MultiplyBy7");
//     expect(
//       await context.web3.eth.estimateGas({
//         from: PRECOMPILE_BATCH_ADDRESS,
//         data: contract.byteCode,
//       })
//     ).to.equal(156994);
//   });
// });

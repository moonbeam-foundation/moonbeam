// import "@moonbeam-network/api-augment";
// import { expect, describeSuite, beforeAll } from "@moonwall/cli";
// import { ALITH_ADDRESS } from "@moonwall/util";
// import { encodeDeployData, encodeFunctionData } from "viem";
// import { TransactionTypes, deployAndCreateCompiledContract } from "../../../helpers/viem.js";
// import { verifyLatestBlockFees } from "../../../helpers/block.js";
// import { createEthersTxn } from "../../../helpers/ethers.js";
// import { customDevRpcRequest } from "../../../helpers/common.js";
// import { before } from "node:test";
// import { getCompiled } from "../../../helpers/contracts.js";

// describeSuite({
//   id: "D0604",
//   title: "Contract event",
//   foundationMethods: "dev",
//   testCases: ({ context, it, log }) => {
//     // let emitterAddress: `0x${string}`;
//     // let emitterAbi: any[];

//     // beforeAll(async () => {
//     //   const { contractAddress, abi } = await deployAndCreateCompiledContract(
//     //     context,
//     //     "EventEmitter"
//     //   );

//     //   emitterAddress = contractAddress;
//     //   emitterAbi = abi;
//     // });

//     for (const txnType of TransactionTypes) {
//       it({
//         id: `T0${TransactionTypes.indexOf(txnType) + 1}`,
//         title: "should contain event",
//         test: async function () {
//           const { byteCode, contract } = getCompiled("EventEmitter");

//           const {rawSigned}= await createEthersTxn(context, {
//             data: encodeDeployData({ abi: contract.abi, bytecode: byteCode , args: []}),
//             txnType
//           });

//           const {result}=await context.createBlock(rawSigned);
//           expect(result?.successful).toBe(true);
//           const receipt = await context.viemClient("public").getTransactionReceipt({hash: result?.hash as `0x${string}`});
//           log(receipt)
//           expect(receipt.logs.length).toBe(1);
//           expect()
//           // const { rawTx } = await createContract(context, "EventEmitter", { from: alith.address });
//           // const { result } = await context.createBlock(rawTx);
//           // const receipt = await context.web3.eth.getTransactionReceipt(result.hash);

//           // expect(receipt.logs.length).to.be.eq(1);
//           // expect(
//           //   "0x" + receipt.logs[0].topics[1].substring(26, receipt.logs[0].topics[1].length + 1)
//           // ).to.be.eq(alith.address.toLowerCase()); // web3 doesn't checksum
//         },
//       });
//     }
//   },
// });
// // describeDevMoonbeamAllEthTxTypes("Contract - Event", (context) => {
// //   it("should contain event", async function () {
// //     const { rawTx } = await createContract(context, "EventEmitter", { from: alith.address });
// //     const { result } = await context.createBlock(rawTx);
// //     const receipt = await context.web3.eth.getTransactionReceipt(result.hash);

// //     expect(receipt.logs.length).to.be.eq(1);
// //     expect(
// //       "0x" + receipt.logs[0].topics[1].substring(26, receipt.logs[0].topics[1].length + 1)
// //     ).to.be.eq(alith.address.toLowerCase()); // web3 doesn't checksum
// //   });
// // });

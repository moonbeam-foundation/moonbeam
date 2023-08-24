// TODO: Moonriver only testcase
// describeDevMoonbeam(
//   "Precompiles - xcm utils",
//   (context) => {
//     it({id:"",title:"moonriver does not allow to execute a custom xcm message",
// test: async function () {
//       let random = generateKeyringPair();

//    const transferCall = context.polkadotJs().tx.balances.transfer(random.address, 1n * GLMR);
//       const transferCallEncoded = transferCall?.method.toHex();

//       const xcmMessage = {
//         V2: [
//           {
//             Transact: {
//               originType: "SovereignAccount",
//               requireWeightAtMost: new BN(525_000_000).add(new BN(100_000_000)),
// 21_000 gas limit
//               call: {
//                 encoded: transferCallEncoded,
//               },
//             },
//           },
//         ],
//       };

//       const receivedMessage: XcmVersionedXcm = context.polkadotJs().createType(
//         "XcmVersionedXcm",
//         xcmMessage
//       ) as any;

//       const { result } = await context.createBlock(
//         createTransaction(context, {
//           ...ALITH_TRANSACTION_TEMPLATE,
//           gas: undefined,
//           to: PRECOMPILE_XCM_UTILS_ADDRESS,
//           data: XCM_UTILSTRANSACTOR_INTERFACE.encodeFunctionData("xcmExecute", [
//             receivedMessage.toU8a(),
//             2_000_000_000,
//           ]),
//         })
//       );
//       expectEVMResult(result.events, "Revert");

//       const revertReason = await extractRevertReason(result.hash, context.ethers);
//       // Full error expected:
//       // Dispatched call failed with error: Module(ModuleError { index: 0, error: [5, 0, 0, 0],
//       //  message: Some("CallFiltered") })
//       expect(revertReason).to.contain("CallFiltered");
//     });
//   },
//   "Legacy",
//   "moonriver"
// );

// TODO: Moonbeam only testcase
// describeDevMoonbeam(
//   "Precompiles - xcm utils",
//   (context) => {
//     it({id:"",title:"moonbeam does not allow to execute a custom xcm message",
// test: async function () {
//       let random = generateKeyringPair();

//   const transferCall = context.polkadotJs().tx.balances.transfer(random.address, 1n * GLMR);
//     const transferCallEncoded = transferCall?.method.toHex();

//       const xcmMessage = {
//         V2: [
//           {
//             Transact: {
//               originType: "SovereignAccount",
//               requireWeightAtMost: new BN(525_000_000).add(new BN(100_000_000)),
// 21_000 gas limit
//               call: {
//                 encoded: transferCallEncoded,
//               },
//             },
//           },
//         ],
//       };

//       const receivedMessage: XcmVersionedXcm = context.polkadotJs().createType(
//         "XcmVersionedXcm",
//         xcmMessage
//       ) as any;

//       const { result } = await context.createBlock(
//         createTransaction(context, {
//           ...ALITH_TRANSACTION_TEMPLATE,
//           gas: undefined,
//           gasPrice: 1_000_000_000_000,
//           to: PRECOMPILE_XCM_UTILS_ADDRESS,
//           data: XCM_UTILSTRANSACTOR_INTERFACE.encodeFunctionData("xcmExecute", [
//             receivedMessage.toU8a(),
//             2_000_000_000,
//           ]),
//         })
//       );
//       expectEVMResult(result.events, "Revert");

//       const revertReason = await extractRevertReason(result.hash, context.ethers);
//       // Full error expected:
//       // Dispatched call failed with error: Module(ModuleError { index: 0, error: [5, 0, 0, 0],
//       // message: Some("CallFiltered") })
//       expect(revertReason).to.contain("CallFiltered");
//     });
//   },
//   "Legacy",
//   "moonbeam"
// );

import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { bnToHex, u8aToHex } from "@polkadot/util";
import { descendOriginFromAddress20 } from "../../../helpers/xcm.js";
import { GLMR, generateKeyringPair } from "@moonwall/util";
import { XcmVersionedXcm } from "@polkadot/types/lookup";
import { expectEVMResult } from "../../../helpers/eth-transactions.js";

// import { u8aToHex, bnToHex } from "@polkadot/util";
// import { expect } from "chai";
// import { ethers } from "ethers";
// import { GLMR, PRECOMPILE_XCM_UTILS_ADDRESS } from "../../util/constants";
// import { getCompiled } from "../../util/contracts";
// import { web3EthCall } from "../../util/providers";
// import { describeSuite, describeDevMoonbeam } from "../../util/setup-dev-tests";
// import { generateKeyringPair } from "../../util/accounts";
// import { BN } from "@polkadot/util";
// import type { XcmVersionedXcm } from "@polkadot/types/lookup";
// import { descendOriginFromAddress20 } from "../../util/xcm";
// import { ALITH_TRANSACTION_TEMPLATE, createTransaction } from "../../util/transactions";
// import { expectEVMResult, extractRevertReason } from "../../util/eth-transactions";

export const CLEAR_ORIGIN_WEIGHT = 5_194_000n;

// const XCM_UTILS_CONTRACT = getCompiled("precompiles/xcm-utils/XcmUtils");
// const XCM_UTILSTRANSACTOR_INTERFACE = new ethers.utils.Interface(XCM_UTILS_CONTRACT.contract.abi);

describeSuite({
  id: "D2578",
  title: "Precompiles - xcm utils",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "allows to retrieve parent-based ML account",
      test: async function () {
        const multilocation: [number, {}[]] = [1, []];
        const expectedAddress = u8aToHex(new Uint8Array([...new TextEncoder().encode("Parent")]))
          .padEnd(42, "0")
          .toLowerCase();

        expect(
          (
            (await context.readPrecompile!({
              precompileName: "XcmUtils",
              functionName: "multilocationToAddress",
              args: [multilocation],
            })) as any
          ).toLowerCase()
        ).to.equal(expectedAddress);
      },
    });

    it({
      id: "T02",
      title: "allows to retrieve parachain-based ML account",
      test: async function () {
        const x2_parachain_asset_enum_selector = "0x00";
        const x2_parachain_id = "000007D0";
        const paraId = context.polkadotJs().createType("ParaId", 2000);

        const multilocation: [number, {}[]] = [
          1,
          // Parachain(2000)
          [x2_parachain_asset_enum_selector + x2_parachain_id],
        ];

        const expectedAddress = u8aToHex(
          new Uint8Array([...new TextEncoder().encode("sibl"), ...paraId.toU8a()])
        ).padEnd(42, "0");

        expect(
          (
            (await context.readPrecompile!({
              precompileName: "XcmUtils",
              functionName: "multilocationToAddress",
              args: [multilocation],
            })) as any
          ).toLowerCase()
        ).to.equal(expectedAddress);
      },
    });

    it({
      id: "T03",
      title: "allows to retrieve generic ML-based derivated account",
      test: async function () {
        const x2_parachain_asset_enum_selector = "0x00";
        const x2_parachain_id = "00000001";

        // Junction::AccountKey20
        const account20EnumSelector = "0x03";
        // [0x01; 20]
        const account20Address = "0101010101010101010101010101010101010101";
        // NetworkId::Any
        const account20NetworkId = "00";

        const multilocation: [number, {}[]] =
          // Destination as multilocation
          [
            // one parent
            1,
            // X2(Parachain(2000), AccountId32(account32Address))
            [
              x2_parachain_asset_enum_selector + x2_parachain_id,
              account20EnumSelector + account20Address + account20NetworkId,
            ],
          ];

        const { descendOriginAddress } = descendOriginFromAddress20(context);
        expect(
          (
            (await context.readPrecompile!({
              precompileName: "XcmUtils",
              functionName: "multilocationToAddress",
              args: [multilocation],
            })) as any
          ).toLowerCase()
        ).toBe(descendOriginAddress);
      },
    });

    it({
      id: "T04",
      title: "allows to retrieve weight of message",
      test: async function () {
        const message = {
          V2: [
            {
              ClearOrigin: null,
            },
          ],
        };
        const xcm = context.polkadotJs().createType("VersionedXcm", message);

        expect(
          await context.readPrecompile!({
            precompileName: "XcmUtils",
            functionName: "weightMessage",
            args: [xcm.toHex()],
          })
        ).to.equal(CLEAR_ORIGIN_WEIGHT);
      },
    });

    it({
      id: "T05",
      title: "allows to retrieve units per second for an asset",
      test: async function () {
        // Junction::PalletInstance(3)
        const x2_pallet_instance_enum_selector = "0x04";
        const x2_instance = "03";

        // This represents X1(PalletInstance(3)))

        // This multilocation represents our native token
        const asset = [
          // zero parents
          0,
          // X1(PalletInstance)
          // PalletInstance: Selector (04) + palconst instance 1 byte (03)
          [x2_pallet_instance_enum_selector + x2_instance],
        ];

        const expectedUnitsPerSecond = 50_000n * 1_000_000_000_000n;

        expect(
          await context.readPrecompile!({
            precompileName: "XcmUtils",
            functionName: "getUnitsPerSecond",
            args: [asset],
          })
        ).to.equal(expectedUnitsPerSecond);
      },
    });

    it({
      id: "T06",
      title: "allows to execute a custom xcm message",
      test: async function () {
        let random = generateKeyringPair();

        const transferCall = context.polkadotJs().tx.balances.transfer(random.address, 1n * GLMR);
        const transferCallEncoded = transferCall?.method.toHex();

        const xcmMessage = {
          V2: [
            {
              Transact: {
                originType: "SovereignAccount",
                requireWeightAtMost: 525_000_000n + 100_000_000n, // 21_000 gas limit
                call: {
                  encoded: transferCallEncoded,
                },
              },
            },
          ],
        };

        const receivedMessage: XcmVersionedXcm = context
          .polkadotJs()
          .createType("XcmVersionedXcm", xcmMessage) as any;

        const rawTxn = await context.writePrecompile!({
          precompileName: "XcmUtils",
          functionName: "xcmExecute",
          args: [receivedMessage.toHex(), 2_000_000_000n],
          rawTxOnly: true,
        });

        const { result } = await context.createBlock(rawTxn);
        expectEVMResult(result!.events, "Succeed");

        const testAccountBalance = (
          await context.polkadotJs().query.system.account(random.address)
        ).data.free.toBigInt();

        expect(testAccountBalance).to.eq(1n * GLMR);
      },
    });
  },
});

// TODO: Moonriver only testcase
// describeDevMoonbeam(
//   "Precompiles - xcm utils",
//   (context) => {
//     it({id:"",title:"moonriver does not allow to execute a custom xcm message", test: async function () {
//       let random = generateKeyringPair();

//       const transferCall = context.polkadotJs().tx.balances.transfer(random.address, 1n * GLMR);
//       const transferCallEncoded = transferCall?.method.toHex();

//       const xcmMessage = {
//         V2: [
//           {
//             Transact: {
//               originType: "SovereignAccount",
//               requireWeightAtMost: new BN(525_000_000).add(new BN(100_000_000)), // 21_000 gas limit
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
//     it({id:"",title:"moonbeam does not allow to execute a custom xcm message", test: async function () {
//       let random = generateKeyringPair();

//       const transferCall = context.polkadotJs().tx.balances.transfer(random.address, 1n * GLMR);
//       const transferCallEncoded = transferCall?.method.toHex();

//       const xcmMessage = {
//         V2: [
//           {
//             Transact: {
//               originType: "SovereignAccount",
//               requireWeightAtMost: new BN(525_000_000).add(new BN(100_000_000)), // 21_000 gas limit
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

// describeSuite({id:"",title:"Precompiles - xcm utils",foundationMethods:"dev", testCases ({context, it ,log}) => {
//   it({id:"",title:"allows to execute a custom xcm evm to evm, but reentrancy forbids", test: async function () {
//     let random = generateKeyringPair();

//     const ethTx = {
//       V1: {
//         gas_limit: 21000,
//         fee_payment: {
//           Auto: {
//             Low: null,
//           },
//         },
//         action: {
//           Call: random.address,
//         },
//         value: 1n * GLMR,
//         input: [],
//         access_list: null,
//       },
//     };
//     const transferCall = context.polkadotJs().tx.ethereumXcm.transact(ethTx as any);
//     const transferCallEncoded = transferCall?.method.toHex();

//     const xcmMessage = {
//       V2: [
//         {
//           Transact: {
//             originType: "SovereignAccount",
//             requireWeightAtMost: new BN(525_000_000).add(new BN(25_000_000)), // 21_000 gas limit
//             call: {
//               encoded: transferCallEncoded,
//             },
//           },
//         },
//       ],
//     };

//     const receivedMessage: XcmVersionedXcm = context.polkadotJs().createType(
//       "XcmVersionedXcm",
//       xcmMessage
//     ) as any;

//     await context.createBlock(
//       createTransaction(context, {
//         ...ALITH_TRANSACTION_TEMPLATE,
//         to: PRECOMPILE_XCM_UTILS_ADDRESS,
//         data: XCM_UTILSTRANSACTOR_INTERFACE.encodeFunctionData("xcmExecute", [
//           receivedMessage.toU8a(),
//           4_000_000_000,
//         ]),
//       })
//     );

//     // Tokens transferred
//     const testAccountBalance = (
//       await context.polkadotJs().query.system.account(random.address)
//     ).data.free.toBigInt();

//     // Transfer did not go through, EVM reentrancy NOT POSSIBLE
//     expect(testAccountBalance).to.eq(0n * GLMR);
//   });
// });

// describeSuite({id:"",title:"Precompiles - xcm utils",foundationMethods:"dev", testCases ({context, it ,log}) => {
//   it({id:"",title:"allows to send a custom xcm message", test: async function () {
//     // Sending it to the relay
//     // { parents:1, Here}
//     const dest = [
//       // one parents
//       1,
//       // Here
//       [],
//     ];

//     const xcmMessage = {
//       V2: [
//         {
//           ClearOrigin: null,
//         },
//       ],
//     };

//     const sentMessage: XcmVersionedXcm = context.polkadotJs().createType(
//       "XcmVersionedXcm",
//       xcmMessage
//     ) as any;

//     const { result } = await context.createBlock(
//       createTransaction(context, {
//         ...ALITH_TRANSACTION_TEMPLATE,
//         to: PRECOMPILE_XCM_UTILS_ADDRESS,
//         data: XCM_UTILSTRANSACTOR_INTERFACE.encodeFunctionData("xcmSend", [
//           dest,
//           sentMessage.toU8a(),
//         ]),
//       })
//     );

//     // Verify the result
//     // Expect success
//     expectEVMResult(result.events, "Succeed");
//   });
// });

// describeSuite({id:"",title:"Precompiles - xcm utils",foundationMethods:"dev", testCases ({context, it ,log}) => {
//   it({id:"",title:"does not allow to self-send a custom xcm message", test: async function () {
//     const ownParaId = (await context.polkadotJs().query.parachainInfo.parachainId()) as any;

//     const x1_parachain_asset_enum_selector = "0x00";

//     const x1_parachain_id = ownParaId.toHex().slice(2);

//     // Sending it here
//     // { parents:0, Here}
//     const destHere: [number, {}[]] = [
//       // one parents
//       0,
//       // Here
//       [],
//     ];

//     // Sending it with the representation of the para as seen by the relay
//     // { parents:0, parachain(0)}
//     const destParaRelayView: [number, {}[]] = [
//       // one parents
//       0,
//       // Parachain(0)
//       [x1_parachain_asset_enum_selector + x1_parachain_id],
//     ];

//     // Sending it with the representation of the para as seen by other paras
//     // { parents:1, parachain(0)}
//     const destParaOtherParaView: [number, {}[]] = [
//       // one parents
//       1,
//       // Parachain(0)
//       [x1_parachain_asset_enum_selector + x1_parachain_id],
//     ];

//     const xcmMessage = {
//       V2: [
//         {
//           ClearOrigin: null,
//         },
//       ],
//     };

//     const sentMessage: XcmVersionedXcm = context.polkadotJs().createType(
//       "XcmVersionedXcm",
//       xcmMessage
//     ) as any;

//     // Try sending it with local view
//     const { result: resultHere } = await context.createBlock(
//       createTransaction(context, {
//         ...ALITH_TRANSACTION_TEMPLATE,
//         to: PRECOMPILE_XCM_UTILS_ADDRESS,
//         data: XCM_UTILSTRANSACTOR_INTERFACE.encodeFunctionData("xcmSend", [
//           destHere,
//           sentMessage.toU8a(),
//         ]),
//       })
//     );

//     // Verify the result
//     // Expect success
//     expectEVMResult(resultHere.events, "Revert");
//     const revertReason = await extractRevertReason(resultHere.hash, context.ethers);
//     // Full error expected:
//     // Dispatched call failed with error: Module(ModuleError { index: 28, error: [0, 0, 0, 0],
//     // message: Some("Unreachable") })
//     expect(revertReason).to.contain("Unreachable");

//     // Try sending it with para relay view
//     const { result: resultParaRelayView } = await context.createBlock(
//       createTransaction(context, {
//         ...ALITH_TRANSACTION_TEMPLATE,
//         to: PRECOMPILE_XCM_UTILS_ADDRESS,
//         data: XCM_UTILSTRANSACTOR_INTERFACE.encodeFunctionData("xcmSend", [
//           destParaRelayView,
//           sentMessage.toU8a(),
//         ]),
//       })
//     );

//     // Verify the result
//     // Expect success
//     expectEVMResult(resultParaRelayView.events, "Revert");
//     const revertReason2 = await extractRevertReason(resultHere.hash, context.ethers);
//     // Full error expected:
//     // Dispatched call failed with error: Module(ModuleError { index: 28, error: [0, 0, 0, 0],
//     // message: Some("Unreachable") })
//     expect(revertReason2).to.contain("Unreachable");

//     // Try sending it with another para view (parents 1)
//     const { result: resultParaOtherParaView } = await context.createBlock(
//       createTransaction(context, {
//         ...ALITH_TRANSACTION_TEMPLATE,
//         to: PRECOMPILE_XCM_UTILS_ADDRESS,
//         data: XCM_UTILSTRANSACTOR_INTERFACE.encodeFunctionData("xcmSend", [
//           destParaOtherParaView,
//           sentMessage.toU8a(),
//         ]),
//       })
//     );

//     // Verify the result
//     // Expect success
//     expectEVMResult(resultParaOtherParaView.events, "Revert");

//     const revertReason3 = await extractRevertReason(resultParaOtherParaView.hash, context.ethers);
//     // Full error expected:
//     // Dispatched call failed with error: Module(ModuleError { index: 28, error: [1, 0, 0, 0],
//     // message: Some("SendFailure") })
//     expect(revertReason3).to.contain("SendFailure");
//   });
// });

import "@moonbeam-network/api-augment";

import { u8aToHex, bnToHex } from "@polkadot/util";
import { expect } from "chai";
import { ethers } from "ethers";
import { GLMR, PRECOMPILE_XCM_UTILS_ADDRESS } from "../../util/constants";
import { getCompiled } from "../../util/contracts";
import { web3EthCall } from "../../util/providers";
import { describeDevMoonbeamAllEthTxTypes, describeDevMoonbeam } from "../../util/setup-dev-tests";
import { generateKeyringPair } from "../../util/accounts";
import { BN } from "@polkadot/util";
import type { XcmVersionedXcm } from "@polkadot/types/lookup";
import { descendOriginFromAddress20 } from "../../util/xcm";
import { ALITH_TRANSACTION_TEMPLATE, createTransaction } from "../../util/transactions";
import { expectEVMResult, extractRevertReason } from "../../util/eth-transactions";

export const CLEAR_ORIGIN_WEIGHT = 5_194_000n;

const XCM_UTILS_CONTRACT = getCompiled("precompiles/xcm-utils/XcmUtils");
const XCM_UTILSTRANSACTOR_INTERFACE = new ethers.utils.Interface(XCM_UTILS_CONTRACT.contract.abi);

describeDevMoonbeamAllEthTxTypes("Precompiles - xcm utils", (context) => {
  it("allows to retrieve parent-based ML account", async function () {
    const multilocation: [number, {}[]] =
      // Destination as multilocation
      [
        // one parent
        1,
        [],
      ];
    const result = await web3EthCall(context.web3, {
      to: PRECOMPILE_XCM_UTILS_ADDRESS,
      data: XCM_UTILSTRANSACTOR_INTERFACE.encodeFunctionData("multilocationToAddress", [
        multilocation,
      ]),
    });

    const expectedAddress = u8aToHex(new Uint8Array([...new TextEncoder().encode("Parent")]))
      .slice(2)
      .padEnd(40, "0");

    expect(result.result).to.equal(`0x${expectedAddress.padStart(64, "0")}`);
  });

  it("allows to retrieve parachain-based ML account", async function () {
    const x2_parachain_asset_enum_selector = "0x00";
    const x2_parachain_id = "000007D0";
    const paraId = context.polkadotApi.createType("ParaId", 2000);

    const multilocation: [number, {}[]] =
      // Destination as multilocation
      [
        // one parent
        1,
        // Parachain(2000)
        [x2_parachain_asset_enum_selector + x2_parachain_id],
      ];

    const result = await web3EthCall(context.web3, {
      to: PRECOMPILE_XCM_UTILS_ADDRESS,
      data: XCM_UTILSTRANSACTOR_INTERFACE.encodeFunctionData("multilocationToAddress", [
        multilocation,
      ]),
    });

    const expectedAddress = u8aToHex(
      new Uint8Array([...new TextEncoder().encode("sibl"), ...paraId.toU8a()])
    )
      .slice(2)
      .padEnd(40, "0");

    expect(result.result).to.equal(`0x${expectedAddress.padStart(64, "0")}`);
  });

  it("allows to retrieve generic ML-based derivated account", async function () {
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

    const result = await web3EthCall(context.web3, {
      to: PRECOMPILE_XCM_UTILS_ADDRESS,
      data: XCM_UTILSTRANSACTOR_INTERFACE.encodeFunctionData("multilocationToAddress", [
        multilocation,
      ]),
    });

    const { originAddress, descendOriginAddress } = descendOriginFromAddress20(context);
    expect(result.result).to.equal(`0x${descendOriginAddress.slice(2).padStart(64, "0")}`);
  });

  it("allows to retrieve weight of message", async function () {
    const message = {
      V2: [
        {
          ClearOrigin: null,
        },
      ],
    };

    const xcm = await context.polkadotApi.createType("VersionedXcm", message);

    const result = await web3EthCall(context.web3, {
      to: PRECOMPILE_XCM_UTILS_ADDRESS,
      data: XCM_UTILSTRANSACTOR_INTERFACE.encodeFunctionData("weightMessage", [xcm.toU8a()]),
    });
    const expectedWeightHex = "0x" + bnToHex(CLEAR_ORIGIN_WEIGHT).slice(2).padStart(64, "0");

    expect(result.result).to.equal(expectedWeightHex);
  });

  it("allows to retrieve units per second for an asset", async function () {
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

    const result = await web3EthCall(context.web3, {
      to: PRECOMPILE_XCM_UTILS_ADDRESS,
      data: XCM_UTILSTRANSACTOR_INTERFACE.encodeFunctionData("getUnitsPerSecond", [asset]),
    });

    const expectedUnitsPerSecond = 50_000n * 1_000_000_000_000n;
    const expectedUnitsHex = "0x" + bnToHex(expectedUnitsPerSecond).slice(2).padStart(64, "0");

    expect(result.result).to.equal(expectedUnitsHex);
  });
});

describeDevMoonbeamAllEthTxTypes("Precompiles - xcm utils", (context) => {
  it("allows to execute a custom xcm message", async function () {
    let random = generateKeyringPair();

    const transferCall = context.polkadotApi.tx.balances.transfer(random.address, 1n * GLMR);
    const transferCallEncoded = transferCall?.method.toHex();

    const xcmMessage = {
      V2: [
        {
          Transact: {
            originType: "SovereignAccount",
            requireWeightAtMost: new BN(525_000_000).add(new BN(100_000_000)), // 21_000 gas limit
            call: {
              encoded: transferCallEncoded,
            },
          },
        },
      ],
    };

    const receivedMessage: XcmVersionedXcm = context.polkadotApi.createType(
      "XcmVersionedXcm",
      xcmMessage
    ) as any;

    await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        gas: undefined,
        to: PRECOMPILE_XCM_UTILS_ADDRESS,
        data: XCM_UTILSTRANSACTOR_INTERFACE.encodeFunctionData("xcmExecute", [
          receivedMessage.toU8a(),
          2_000_000_000,
        ]),
      })
    );

    // Tokens transferred
    const testAccountBalance = (
      await context.polkadotApi.query.system.account(random.address)
    ).data.free.toBigInt();

    expect(testAccountBalance).to.eq(1n * GLMR);
  });
});

describeDevMoonbeam(
  "Precompiles - xcm utils",
  (context) => {
    it("moonriver does not allow to execute a custom xcm message", async function () {
      let random = generateKeyringPair();

      const transferCall = context.polkadotApi.tx.balances.transfer(random.address, 1n * GLMR);
      const transferCallEncoded = transferCall?.method.toHex();

      const xcmMessage = {
        V2: [
          {
            Transact: {
              originType: "SovereignAccount",
              requireWeightAtMost: new BN(525_000_000).add(new BN(100_000_000)), // 21_000 gas limit
              call: {
                encoded: transferCallEncoded,
              },
            },
          },
        ],
      };

      const receivedMessage: XcmVersionedXcm = context.polkadotApi.createType(
        "XcmVersionedXcm",
        xcmMessage
      ) as any;

      const { result } = await context.createBlock(
        createTransaction(context, {
          ...ALITH_TRANSACTION_TEMPLATE,
          gas: undefined,
          to: PRECOMPILE_XCM_UTILS_ADDRESS,
          data: XCM_UTILSTRANSACTOR_INTERFACE.encodeFunctionData("xcmExecute", [
            receivedMessage.toU8a(),
            2_000_000_000,
          ]),
        })
      );
      expectEVMResult(result.events, "Revert");

      const revertReason = await extractRevertReason(result.hash, context.ethers);
      // Full error expected:
      // Dispatched call failed with error: Module(ModuleError { index: 0, error: [5, 0, 0, 0],
      //  message: Some("CallFiltered") })
      expect(revertReason).to.contain("CallFiltered");
    });
  },
  "Legacy",
  "moonriver"
);

describeDevMoonbeam(
  "Precompiles - xcm utils",
  (context) => {
    it("moonbeam does not allow to execute a custom xcm message", async function () {
      let random = generateKeyringPair();

      const transferCall = context.polkadotApi.tx.balances.transfer(random.address, 1n * GLMR);
      const transferCallEncoded = transferCall?.method.toHex();

      const xcmMessage = {
        V2: [
          {
            Transact: {
              originType: "SovereignAccount",
              requireWeightAtMost: new BN(525_000_000).add(new BN(100_000_000)), // 21_000 gas limit
              call: {
                encoded: transferCallEncoded,
              },
            },
          },
        ],
      };

      const receivedMessage: XcmVersionedXcm = context.polkadotApi.createType(
        "XcmVersionedXcm",
        xcmMessage
      ) as any;

      const { result } = await context.createBlock(
        createTransaction(context, {
          ...ALITH_TRANSACTION_TEMPLATE,
          gas: undefined,
          gasPrice: 1_000_000_000_000,
          to: PRECOMPILE_XCM_UTILS_ADDRESS,
          data: XCM_UTILSTRANSACTOR_INTERFACE.encodeFunctionData("xcmExecute", [
            receivedMessage.toU8a(),
            2_000_000_000,
          ]),
        })
      );
      expectEVMResult(result.events, "Revert");

      const revertReason = await extractRevertReason(result.hash, context.ethers);
      // Full error expected:
      // Dispatched call failed with error: Module(ModuleError { index: 0, error: [5, 0, 0, 0],
      // message: Some("CallFiltered") })
      expect(revertReason).to.contain("CallFiltered");
    });
  },
  "Legacy",
  "moonbeam"
);

describeDevMoonbeamAllEthTxTypes("Precompiles - xcm utils", (context) => {
  it("allows to execute a custom xcm evm to evm, but reentrancy forbids", async function () {
    let random = generateKeyringPair();

    const ethTx = {
      V1: {
        gas_limit: 21000,
        fee_payment: {
          Auto: {
            Low: null,
          },
        },
        action: {
          Call: random.address,
        },
        value: 1n * GLMR,
        input: [],
        access_list: null,
      },
    };
    const transferCall = context.polkadotApi.tx.ethereumXcm.transact(ethTx as any);
    const transferCallEncoded = transferCall?.method.toHex();

    const xcmMessage = {
      V2: [
        {
          Transact: {
            originType: "SovereignAccount",
            requireWeightAtMost: new BN(525_000_000).add(new BN(25_000_000)), // 21_000 gas limit
            call: {
              encoded: transferCallEncoded,
            },
          },
        },
      ],
    };

    const receivedMessage: XcmVersionedXcm = context.polkadotApi.createType(
      "XcmVersionedXcm",
      xcmMessage
    ) as any;

    await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_XCM_UTILS_ADDRESS,
        data: XCM_UTILSTRANSACTOR_INTERFACE.encodeFunctionData("xcmExecute", [
          receivedMessage.toU8a(),
          4_000_000_000,
        ]),
      })
    );

    // Tokens transferred
    const testAccountBalance = (
      await context.polkadotApi.query.system.account(random.address)
    ).data.free.toBigInt();

    // Transfer did not go through, EVM reentrancy NOT POSSIBLE
    expect(testAccountBalance).to.eq(0n * GLMR);
  });
});

describeDevMoonbeamAllEthTxTypes("Precompiles - xcm utils", (context) => {
  it("allows to send a custom xcm message", async function () {
    // Sending it to the relay
    // { parents:1, Here}
    const dest = [
      // one parents
      1,
      // Here
      [],
    ];

    const xcmMessage = {
      V2: [
        {
          ClearOrigin: null,
        },
      ],
    };

    const sentMessage: XcmVersionedXcm = context.polkadotApi.createType(
      "XcmVersionedXcm",
      xcmMessage
    ) as any;

    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_XCM_UTILS_ADDRESS,
        data: XCM_UTILSTRANSACTOR_INTERFACE.encodeFunctionData("xcmSend", [
          dest,
          sentMessage.toU8a(),
        ]),
      })
    );

    // Verify the result
    // Expect success
    expectEVMResult(result.events, "Succeed");
  });
});

describeDevMoonbeamAllEthTxTypes("Precompiles - xcm utils", (context) => {
  it("does not allow to self-send a custom xcm message", async function () {
    const ownParaId = (await context.polkadotApi.query.parachainInfo.parachainId()) as any;

    const x1_parachain_asset_enum_selector = "0x00";

    const x1_parachain_id = ownParaId.toHex().slice(2);

    // Sending it here
    // { parents:0, Here}
    const destHere: [number, {}[]] = [
      // one parents
      0,
      // Here
      [],
    ];

    // Sending it with the representation of the para as seen by the relay
    // { parents:0, parachain(0)}
    const destParaRelayView: [number, {}[]] = [
      // one parents
      0,
      // Parachain(0)
      [x1_parachain_asset_enum_selector + x1_parachain_id],
    ];

    // Sending it with the representation of the para as seen by other paras
    // { parents:1, parachain(0)}
    const destParaOtherParaView: [number, {}[]] = [
      // one parents
      1,
      // Parachain(0)
      [x1_parachain_asset_enum_selector + x1_parachain_id],
    ];

    const xcmMessage = {
      V2: [
        {
          ClearOrigin: null,
        },
      ],
    };

    const sentMessage: XcmVersionedXcm = context.polkadotApi.createType(
      "XcmVersionedXcm",
      xcmMessage
    ) as any;

    // Try sending it with local view
    const { result: resultHere } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_XCM_UTILS_ADDRESS,
        data: XCM_UTILSTRANSACTOR_INTERFACE.encodeFunctionData("xcmSend", [
          destHere,
          sentMessage.toU8a(),
        ]),
      })
    );

    // Verify the result
    // Expect success
    expectEVMResult(resultHere.events, "Revert");
    const revertReason = await extractRevertReason(resultHere.hash, context.ethers);
    // Full error expected:
    // Dispatched call failed with error: Module(ModuleError { index: 28, error: [0, 0, 0, 0],
    // message: Some("Unreachable") })
    expect(revertReason).to.contain("Unreachable");

    // Try sending it with para relay view
    const { result: resultParaRelayView } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_XCM_UTILS_ADDRESS,
        data: XCM_UTILSTRANSACTOR_INTERFACE.encodeFunctionData("xcmSend", [
          destParaRelayView,
          sentMessage.toU8a(),
        ]),
      })
    );

    // Verify the result
    // Expect success
    expectEVMResult(resultParaRelayView.events, "Revert");
    const revertReason2 = await extractRevertReason(resultHere.hash, context.ethers);
    // Full error expected:
    // Dispatched call failed with error: Module(ModuleError { index: 28, error: [0, 0, 0, 0],
    // message: Some("Unreachable") })
    expect(revertReason2).to.contain("Unreachable");

    // Try sending it with another para view (parents 1)
    const { result: resultParaOtherParaView } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_XCM_UTILS_ADDRESS,
        data: XCM_UTILSTRANSACTOR_INTERFACE.encodeFunctionData("xcmSend", [
          destParaOtherParaView,
          sentMessage.toU8a(),
        ]),
      })
    );

    // Verify the result
    // Expect success
    expectEVMResult(resultParaOtherParaView.events, "Revert");

    const revertReason3 = await extractRevertReason(resultParaOtherParaView.hash, context.ethers);
    // Full error expected:
    // Dispatched call failed with error: Module(ModuleError { index: 28, error: [1, 0, 0, 0],
    // message: Some("SendFailure") })
    expect(revertReason3).to.contain("SendFailure");
  });
});

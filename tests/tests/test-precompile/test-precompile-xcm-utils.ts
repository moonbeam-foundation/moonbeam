import "@moonbeam-network/api-augment";

import { u8aToHex, bnToHex } from "@polkadot/util";
import { expect } from "chai";
import { ethers } from "ethers";
import { PRECOMPILE_XCM_UTILS_ADDRESS } from "../../util/constants";
import { getCompiled } from "../../util/contracts";
import { web3EthCall } from "../../util/providers";
import { describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";

import { descendOriginFromAddress } from "../../util/xcm";

export const CLEAR_ORIGIN_WEIGHT = 8_268_000n;

const XCM_UTILS_CONTRACT = getCompiled("XcmUtils");
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

    const { originAddress, descendOriginAddress } = descendOriginFromAddress(context);
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

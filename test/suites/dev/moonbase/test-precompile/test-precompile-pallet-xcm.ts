import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, fetchCompiledContract, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, BALTATHAR_ADDRESS, alith, createEthersTransaction } from "@moonwall/util";
import { numberToHex } from "@polkadot/util";
import { encodeFunctionData, erc20Abi } from "viem";
import {
  expectEVMResult,
  mockAssetBalance,
  registerForeignAsset,
  relayAssetMetadata,
  ARBITRARY_ASSET_ID,
  RELAY_SOURCE_LOCATION_V4,
  registerAndFundAsset,
  PARA_1000_SOURCE_LOCATION,
} from "../../../../helpers";
import { ethers } from "ethers";
import { para1000AssetMetadata } from "./test-precompile-pallet-xcm-2";
const PRECOMPILE_PALLET_XCM_ADDRESS: `0x${string}` = "0x000000000000000000000000000000000000081A";

describeSuite({
  id: "D012900",
  title: "Precompiles - PalletXcm",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let foreignAssetContract: ethers.Contract;
    const amountToSend = 100n;

    beforeAll(async () => {
      const someBalance = 100_000_000_000_000_000_000_000_000n;

      // Register the asset
      const { registeredAssetId, contractAddress } = await registerAndFundAsset(
        context,
        {
          id: ARBITRARY_ASSET_ID,
          location: RELAY_SOURCE_LOCATION_V4,
          metadata: relayAssetMetadata,
          relativePrice: 1_000_000_000_000_000_000n,
        },
        someBalance,
        ALITH_ADDRESS,
        false
      );

      console.log("contract address: ", contractAddress);
      console.log("asset id: ", registeredAssetId);

      foreignAssetContract = new ethers.Contract(contractAddress, erc20Abi, context.ethers());
    });

    it({
      id: "T01",
      title: "allows to call transferAssetsLocation function",
      test: async function () {
        const { abi: xcmInterface } = fetchCompiledContract("XCM");
        const assetBalanceBefore = await foreignAssetContract.balanceOf(ALITH_ADDRESS);

        console.log("BALANCE BEFORE: ", assetBalanceBefore);

        const dest: [number, any[]] = [1, []];

        const destinationAddress =
          "0101010101010101010101010101010101010101010101010101010101010101";
        const destinationNetworkId = "00";
        const beneficiary: [number, any[]] = [
          0,
          // junction: AccountId32 enum (01) + the 32 byte account + Any network selector(00)
          ["0x01" + destinationAddress + destinationNetworkId],
        ];

        const assetLocation: [number, any[]] = [1, []];
        const assetLocationInfo = [[assetLocation, amountToSend]];

        const rawTxn = await createEthersTransaction(context, {
          to: PRECOMPILE_PALLET_XCM_ADDRESS,
          data: encodeFunctionData({
            abi: xcmInterface,
            args: [dest, beneficiary, assetLocationInfo, 0],
            functionName: "transferAssetsLocation",
          }),
          gasLimit: 500_000n,
        });

        const result = await context.createBlock(rawTxn);
        expectEVMResult(result.result!.events, "Succeed");

        const assetBalanceAfter = await foreignAssetContract.balanceOf(ALITH_ADDRESS);
        expect(assetBalanceAfter).to.equal(assetBalanceBefore - amountToSend);
      },
    });

    it({
      id: "T02",
      title: "allows to call transferAssetsToPara20 function",
      test: async function () {
        const { abi: xcmInterface } = fetchCompiledContract("XCM");
        const assetBalanceBefore = await foreignAssetContract.balanceOf(ALITH_ADDRESS);

        const paraId = 1000n;
        const assetAddressInfo = [[await foreignAssetContract.getAddress(), amountToSend]];

        const rawTxn = await createEthersTransaction(context, {
          to: PRECOMPILE_PALLET_XCM_ADDRESS,
          data: encodeFunctionData({
            abi: xcmInterface,
            args: [paraId, BALTATHAR_ADDRESS, assetAddressInfo, 0],
            functionName: "transferAssetsToPara20",
          }),
          gasLimit: 500_000n,
        });

        const result = await context.createBlock(rawTxn);
        expectEVMResult(result.result!.events, "Succeed");

        const assetBalanceAfter = await foreignAssetContract.balanceOf(ALITH_ADDRESS);
        expect(assetBalanceAfter).to.equal(assetBalanceBefore - amountToSend);
      },
    });

    it({
      id: "T03",
      title: "allows to call transferAssetsToPara32 function",
      test: async function () {
        const { abi: xcmInterface } = fetchCompiledContract("XCM");
        const assetBalanceBefore = await foreignAssetContract.balanceOf(ALITH_ADDRESS);

        const paraId = 1000n;
        const assetAddressInfo = [[await foreignAssetContract.getAddress(), amountToSend]];
        const beneficiaryAddress = "01010101010101010101010101010101";

        const rawTxn = await createEthersTransaction(context, {
          to: PRECOMPILE_PALLET_XCM_ADDRESS,
          data: encodeFunctionData({
            abi: xcmInterface,
            args: [paraId, beneficiaryAddress, assetAddressInfo, 0],
            functionName: "transferAssetsToPara32",
          }),
          gasLimit: 500_000n,
        });

        const result = await context.createBlock(rawTxn);
        expectEVMResult(result.result!.events, "Succeed");

        const assetBalanceAfter = await foreignAssetContract.balanceOf(ALITH_ADDRESS);
        expect(assetBalanceAfter).to.equal(assetBalanceBefore - amountToSend);
      },
    });

    it({
      id: "T04",
      title: "allows to call transferAssetsToRelay function",
      test: async function () {
        const { abi: xcmInterface } = fetchCompiledContract("XCM");
        const assetBalanceBefore = await foreignAssetContract.balanceOf(ALITH_ADDRESS);

        const assetAddressInfo = [[await foreignAssetContract.getAddress(), amountToSend]];
        const beneficiaryAddress = "01010101010101010101010101010101";

        const rawTxn = await createEthersTransaction(context, {
          to: PRECOMPILE_PALLET_XCM_ADDRESS,
          data: encodeFunctionData({
            abi: xcmInterface,
            args: [beneficiaryAddress, assetAddressInfo, 0],
            functionName: "transferAssetsToRelay",
          }),
          gasLimit: 500_000n,
        });

        const result = await context.createBlock(rawTxn);
        expectEVMResult(result.result!.events, "Succeed");

        const assetBalanceAfter = await foreignAssetContract.balanceOf(ALITH_ADDRESS);
        expect(assetBalanceAfter).to.equal(assetBalanceBefore - amountToSend);
      },
    });

    it({
      id: "T05",
      title: "allows to call transferAssetsUsingTypeAndThenLocation::8425d893 selector",
      test: async function () {
        const { abi: xcmInterface } = fetchCompiledContract("XCM");
        const assetBalanceBefore = await foreignAssetContract.balanceOf(ALITH_ADDRESS);

        const dest: [number, any[]] = [1, []];
        const assetLocation: [number, any[]] = [1, []];
        const assetLocationInfo = [[assetLocation, amountToSend]];

        // DestinationReserve
        const assetsAndFeesTransferType = 2;

        const message = {
          V3: [
            {
              ClearOrigin: null,
            },
          ],
        };
        const xcmOnDest = context.polkadotJs().createType("XcmVersionedXcm", message);

        const rawTxn = await createEthersTransaction(context, {
          to: PRECOMPILE_PALLET_XCM_ADDRESS,
          data: encodeFunctionData({
            abi: xcmInterface,
            args: [
              dest,
              assetLocationInfo,
              assetsAndFeesTransferType,
              0n,
              assetsAndFeesTransferType,
              xcmOnDest.toHex(),
            ],
            functionName: "transferAssetsUsingTypeAndThenLocation",
          }),
          gasLimit: 500_000n,
        });

        const result = await context.createBlock(rawTxn);
        expectEVMResult(result.result!.events, "Succeed");

        const assetBalanceAfter = await foreignAssetContract.balanceOf(ALITH_ADDRESS);
        expect(assetBalanceAfter).to.equal(assetBalanceBefore - amountToSend);
      },
    });

    it({
      id: "T06",
      title: "allows to call transferAssetsUsingTypeAndThenLocation::fc19376c selector",
      test: async function () {
        const { abi: xcmInterface } = fetchCompiledContract("XCM");
        const assetBalanceBefore = await foreignAssetContract.balanceOf(ALITH_ADDRESS);

        const paraIdInHex = numberToHex(2000, 32);
        const parachain_enum_selector = "0x00";

        // This represents X2(Parent, Parachain(2000))
        const dest: [number, any[]] = [1, [parachain_enum_selector + paraIdInHex.slice(2)]];

        const remoteReserve: [number, any[]] = [1, []];
        const assetLocation: [number, any[]] = [1, []];
        const assetLocationInfo = [[assetLocation, amountToSend]];

        const message = {
          V3: [
            {
              ClearOrigin: null,
            },
          ],
        };
        const xcmOnDest = context.polkadotJs().createType("XcmVersionedXcm", message);

        const rawTxn = await createEthersTransaction(context, {
          to: PRECOMPILE_PALLET_XCM_ADDRESS,
          data: encodeFunctionData({
            abi: xcmInterface,
            args: [dest, assetLocationInfo, 0n, xcmOnDest.toHex(), remoteReserve],
            functionName: "transferAssetsUsingTypeAndThenLocation",
          }),
          gasLimit: 500_000n,
        });

        const result = await context.createBlock(rawTxn);
        expectEVMResult(result.result!.events, "Succeed");

        const assetBalanceAfter = await foreignAssetContract.balanceOf(ALITH_ADDRESS);
        expect(assetBalanceAfter).to.equal(assetBalanceBefore - amountToSend);
      },
    });

    it({
      id: "T07",
      title: "allows to call transferAssetsUsingTypeAndThenAddress::998093ee selector",
      test: async function () {
        const { abi: xcmInterface } = fetchCompiledContract("XCM");
        const assetBalanceBefore = await foreignAssetContract.balanceOf(ALITH_ADDRESS);

        // Relay as destination
        const dest: [number, any[]] = [1, []];
        const assetAddressInfo = [[await foreignAssetContract.getAddress(), amountToSend]];

        // DestinationReserve
        const assetsAndFeesTransferType = 2;

        const message = {
          V3: [
            {
              ClearOrigin: null,
            },
          ],
        };
        const xcmOnDest = context.polkadotJs().createType("XcmVersionedXcm", message);

        const rawTxn = await createEthersTransaction(context, {
          to: PRECOMPILE_PALLET_XCM_ADDRESS,
          data: encodeFunctionData({
            abi: xcmInterface,
            args: [
              dest,
              assetAddressInfo,
              assetsAndFeesTransferType,
              0n,
              assetsAndFeesTransferType,
              xcmOnDest.toHex(),
            ],
            functionName: "transferAssetsUsingTypeAndThenAddress",
          }),
          gasLimit: 500_000n,
        });

        const result = await context.createBlock(rawTxn);
        expectEVMResult(result.result!.events, "Succeed");

        const assetBalanceAfter = await foreignAssetContract.balanceOf(ALITH_ADDRESS);
        expect(assetBalanceAfter).to.equal(assetBalanceBefore - amountToSend);
      },
    });

    it({
      id: "T08",
      title: "allows to call transferAssetsUsingTypeAndThenAddress::aaecfc62 selector",
      test: async function () {
        const { abi: xcmInterface } = fetchCompiledContract("XCM");
        const assetBalanceBefore = await foreignAssetContract.balanceOf(ALITH_ADDRESS);

        const paraIdInHex = numberToHex(2000, 32);
        const parachain_enum_selector = "0x00";

        // This represents X2(Parent, Parachain(2000))
        const dest: [number, any[]] = [1, [parachain_enum_selector + paraIdInHex.slice(2)]];
        const assetAddressInfo = [[await foreignAssetContract.getAddress(), amountToSend]];
        const remoteReserve: [number, any[]] = [1, []];

        const message = {
          V3: [
            {
              ClearOrigin: null,
            },
          ],
        };
        const xcmOnDest = context.polkadotJs().createType("XcmVersionedXcm", message);

        const rawTxn = await createEthersTransaction(context, {
          to: PRECOMPILE_PALLET_XCM_ADDRESS,
          data: encodeFunctionData({
            abi: xcmInterface,
            args: [dest, assetAddressInfo, 0n, xcmOnDest.toHex(), remoteReserve],
            functionName: "transferAssetsUsingTypeAndThenAddress",
          }),
          gasLimit: 500_000n,
        });

        const result = await context.createBlock(rawTxn);
        expectEVMResult(result.result!.events, "Succeed");

        const assetBalanceAfter = await foreignAssetContract.balanceOf(ALITH_ADDRESS);
        expect(assetBalanceAfter).to.equal(assetBalanceBefore - amountToSend);
      },
    });
  },
});

import "@moonbeam-network/api-augment";
import {
  beforeAll,
  describeSuite,
  fetchCompiledContract,
  expect,
  customDevRpcRequest,
} from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  BALTATHAR_ADDRESS,
  alith,
  baltathar,
  createEthersTransaction,
  PRECOMPILE_NATIVE_ERC20_ADDRESS,
} from "@moonwall/util";
import { encodeFunctionData, erc20Abi } from "viem";
import {
  expectEVMResult,
  PARA_1000_SOURCE_LOCATION,
  mockHrmpChannelExistanceTx,
  ARBITRARY_ASSET_ID,
  mockAssetBalance,
  registerForeignAsset,
  relayAssetMetadata,
  RELAY_SOURCE_LOCATION_V4,
  registerAndFundAsset,
} from "../../../../helpers";
import type { AssetMetadata } from "../../../../helpers";
import { ethers } from "ethers";

const PRECOMPILE_PALLET_XCM_ADDRESS: `0x${string}` = "0x000000000000000000000000000000000000081A";

export const para1000AssetMetadata: AssetMetadata = {
  name: "PARA",
  symbol: "PARA",
  decimals: 12n,
  isFrozen: false,
};

describeSuite({
  id: "D012901",
  title: "Precompiles - PalletXcm: Native fee",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let foreignRelayAssetContract: ethers.Contract;
    let foreignParaAssetContract: ethers.Contract;

    const destinationPara = 1000;
    const amountToSend = 100n;

    beforeAll(async () => {
      const balance = 200000000000000n;

      // Register the asset
      const { registeredAssetId: relayAssetId, contractAddress: relayAssetAddress } =
        await registerAndFundAsset(
          context,
          {
            id: ARBITRARY_ASSET_ID,
            location: RELAY_SOURCE_LOCATION_V4,
            metadata: relayAssetMetadata,
            relativePrice: 1_000_000_000_000_000_000n,
          },
          balance,
          ALITH_ADDRESS,
          false
        );

      console.log("Foreign Relay asset address: ", relayAssetAddress);
      console.log("Foreign Relay asset id: ", relayAssetId);

      foreignRelayAssetContract = new ethers.Contract(
        relayAssetAddress,
        erc20Abi,
        context.ethers()
      );

      // Register the asset
      const { registeredAssetId: paraAssetId, contractAddress: paraAssetAddress } =
        await registerAndFundAsset(
          context,
          {
            id: ARBITRARY_ASSET_ID + 1n,
            location: PARA_1000_SOURCE_LOCATION,
            metadata: para1000AssetMetadata,
            relativePrice: 1_000_000_000_000_000_000n,
          },
          balance,
          ALITH_ADDRESS
        );

      console.log("Foreign Para asset address: ", paraAssetAddress);
      console.log("Foreign Para asset id: ", paraAssetId);

      foreignParaAssetContract = new ethers.Contract(paraAssetAddress, erc20Abi, context.ethers());

      // Change the sudo key so that we avoid nonce issues.
      const sudoKeyTx = context.polkadotJs().tx.sudo.setKey(baltathar.address);
      await context.createBlock(await sudoKeyTx.signAsync(alith), { allowFailures: false });
    });

    it({
      id: "T01",
      title: "transferAssetsLocation: allows to pay fees with native asset",
      test: async function () {
        const { abi: xcmInterface } = fetchCompiledContract("XCM");
        const assetBalanceBefore = await foreignRelayAssetContract.balanceOf(ALITH_ADDRESS);

        const dest: [number, any[]] = [1, []];

        const destinationAddress =
          "0101010101010101010101010101010101010101010101010101010101010101";
        const destinationNetworkId = "00";
        const beneficiary: [number, any[]] = [
          0,
          // junction: AccountId32 enum (01) + the 32 byte account + Any network selector(00)
          ["0x01" + destinationAddress + destinationNetworkId],
        ];

        const x1_pallet_instance_enum_selector = "0x04";
        const x1_instance = "03";

        // This multilocation represents our native token
        const nativeAssetLocation = [
          // zero parents
          0,
          // X1(PalletInstance)
          // PalletInstance: Selector (04) + balances pallet instance 1 byte (03)
          [x1_pallet_instance_enum_selector + x1_instance],
        ];

        const nonFeeAssetLocation: [number, any[]] = [1, []];
        const assetLocationInfo = [
          [nonFeeAssetLocation, amountToSend],
          [nativeAssetLocation, 100n],
        ];

        const rawTxn = await createEthersTransaction(context, {
          to: PRECOMPILE_PALLET_XCM_ADDRESS,
          data: encodeFunctionData({
            abi: xcmInterface,
            args: [dest, beneficiary, assetLocationInfo, 1],
            functionName: "transferAssetsLocation",
          }),
          gasLimit: 500_000n,
        });

        const result = await context.createBlock(rawTxn);
        expectEVMResult(result.result!.events, "Succeed");

        const assetBalanceAfter = await foreignRelayAssetContract.balanceOf(ALITH_ADDRESS);
        expect(assetBalanceAfter).to.equal(assetBalanceBefore - amountToSend);
      },
    });

    it({
      id: "T02",
      title: "transferAssetsToPara20: allows to pay fees with native asset",
      test: async function () {
        const { abi: xcmInterface } = fetchCompiledContract("XCM");
        const assetBalanceBefore = await foreignParaAssetContract.balanceOf(ALITH_ADDRESS);

        const paraId = destinationPara;

        // Assets must be sorted, so we put the native one first as it has a lower "parents" field.
        const assetAddressInfo = [
          [PRECOMPILE_NATIVE_ERC20_ADDRESS, amountToSend],
          [await foreignParaAssetContract.getAddress(), amountToSend],
        ];

        const mockHrmp1000Tx = context
          .polkadotJs()
          .tx.sudo.sudo(mockHrmpChannelExistanceTx(context, destinationPara, 1000, 102400, 102400));

        const alithNonce = (
          await context.polkadotJs().query.system.account(alith.address)
        ).nonce.toNumber();
        const rawTxn = await createEthersTransaction(context, {
          to: PRECOMPILE_PALLET_XCM_ADDRESS,
          data: encodeFunctionData({
            abi: xcmInterface,
            args: [paraId, BALTATHAR_ADDRESS, assetAddressInfo, 0],
            functionName: "transferAssetsToPara20",
          }),
          gasLimit: 500_000n,
          nonce: alithNonce,
        });

        // Insert the two txs in the same block.
        await mockHrmp1000Tx.signAndSend(baltathar);
        await customDevRpcRequest("eth_sendRawTransaction", [rawTxn]);
        await context.createBlock();

        const events = await context.polkadotJs().query.system.events();
        expectEVMResult(events, "Succeed");

        const assetBalanceAfter = await foreignParaAssetContract.balanceOf(ALITH_ADDRESS);
        expect(assetBalanceAfter).to.equal(assetBalanceBefore - amountToSend);
      },
    });

    it({
      id: "T03",
      title: "transferAssetsToPara32: allows to pay fees with native asset",
      test: async function () {
        const { abi: xcmInterface } = fetchCompiledContract("XCM");
        const assetBalanceBefore = await foreignParaAssetContract.balanceOf(ALITH_ADDRESS);

        const paraId = destinationPara;

        // Assets must be sorted, so we put the native one first as it has a lower "parents" field.
        const assetAddressInfo = [
          [PRECOMPILE_NATIVE_ERC20_ADDRESS, amountToSend],
          [await foreignParaAssetContract.getAddress(), amountToSend],
        ];

        const mockHrmp1000Tx = context
          .polkadotJs()
          .tx.sudo.sudo(mockHrmpChannelExistanceTx(context, destinationPara, 1000, 102400, 102400));

        // 32 bytes beneficiary
        const beneficiaryAddress = "01010101010101010101010101010101";

        const alithNonce = (
          await context.polkadotJs().query.system.account(alith.address)
        ).nonce.toNumber();
        const rawTxn = await createEthersTransaction(context, {
          to: PRECOMPILE_PALLET_XCM_ADDRESS,
          data: encodeFunctionData({
            abi: xcmInterface,
            args: [paraId, beneficiaryAddress, assetAddressInfo, 0],
            functionName: "transferAssetsToPara32",
          }),
          gasLimit: 500_000n,
          nonce: alithNonce,
        });

        // Insert the two txs in the same block.
        // First one with baltathar as sudo.
        await mockHrmp1000Tx.signAndSend(baltathar);
        await customDevRpcRequest("eth_sendRawTransaction", [rawTxn]);
        await context.createBlock();

        const events = await context.polkadotJs().query.system.events();
        expectEVMResult(events, "Succeed");

        const assetBalanceAfter = await foreignParaAssetContract.balanceOf(ALITH_ADDRESS);
        expect(assetBalanceAfter).to.equal(assetBalanceBefore - amountToSend);
      },
    });

    it({
      id: "T04",
      title: "transferAssetsToRelay: allows to pay fees with native asset",
      test: async function () {
        const { abi: xcmInterface } = fetchCompiledContract("XCM");
        const assetBalanceBefore = await foreignRelayAssetContract.balanceOf(ALITH_ADDRESS);

        // Assets must be sorted, so we put the native one first as it has a lower "parents" field.
        const assetAddressInfo = [
          [PRECOMPILE_NATIVE_ERC20_ADDRESS, amountToSend],
          [await foreignRelayAssetContract.getAddress(), amountToSend],
        ];

        const mockHrmp1000Tx = context
          .polkadotJs()
          .tx.sudo.sudo(mockHrmpChannelExistanceTx(context, destinationPara, 1000, 102400, 102400));

        // 32 bytes beneficiary
        const beneficiaryAddress = "01010101010101010101010101010101";

        const alithNonce = (
          await context.polkadotJs().query.system.account(alith.address)
        ).nonce.toNumber();
        const rawTxn = await createEthersTransaction(context, {
          to: PRECOMPILE_PALLET_XCM_ADDRESS,
          data: encodeFunctionData({
            abi: xcmInterface,
            args: [beneficiaryAddress, assetAddressInfo, 0],
            functionName: "transferAssetsToRelay",
          }),
          gasLimit: 500_000n,
          nonce: alithNonce,
        });

        // Insert the two txs in the same block.
        // First one with baltathar as sudo.
        await mockHrmp1000Tx.signAndSend(baltathar);
        await customDevRpcRequest("eth_sendRawTransaction", [rawTxn]);
        await context.createBlock();

        const events = await context.polkadotJs().query.system.events();
        expectEVMResult(events, "Succeed");

        const assetBalanceAfter = await foreignRelayAssetContract.balanceOf(ALITH_ADDRESS);
        expect(assetBalanceAfter).to.equal(assetBalanceBefore - amountToSend);
      },
    });

    it({
      id: "T05",
      title:
        "transferAssetsUsingTypeAndThenLocation (8425d893): allows to pay fees with native asset",
      test: async function () {
        const { abi: xcmInterface } = fetchCompiledContract("XCM");
        const assetBalanceBefore = await foreignRelayAssetContract.balanceOf(ALITH_ADDRESS);

        const dest: [number, any[]] = [1, []];
        const assetLocation: [number, any[]] = [1, []];

        const x1_pallet_instance_enum_selector = "0x04";
        const x1_instance = "03";

        const nativeAssetLocation = [
          // zero parents
          0,
          // X1(PalletInstance)
          // PalletInstance: Selector (04) + balances pallet instance 1 byte (03)
          [x1_pallet_instance_enum_selector + x1_instance],
        ];

        const assetLocationInfo = [
          [nativeAssetLocation, amountToSend],
          [assetLocation, amountToSend],
        ];

        // LocalReserve
        const feesTransferType = 1;

        // DestinationReserve
        const assetsTransferType = 2;

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
              assetsTransferType,
              0n,
              feesTransferType,
              xcmOnDest.toHex(),
            ],
            functionName: "transferAssetsUsingTypeAndThenLocation",
          }),
          gasLimit: 500_000n,
        });

        const result = await context.createBlock(rawTxn);
        expectEVMResult(result.result!.events, "Succeed");

        const assetBalanceAfter = await foreignRelayAssetContract.balanceOf(ALITH_ADDRESS);
        expect(assetBalanceAfter).to.equal(assetBalanceBefore - amountToSend);
      },
    });

    it({
      id: "T06",
      title:
        "transferAssetsUsingTypeAndThenAddress (8425d893): allows to pay fees with native asset",
      test: async function () {
        const { abi: xcmInterface } = fetchCompiledContract("XCM");
        const assetBalanceBefore = await foreignRelayAssetContract.balanceOf(ALITH_ADDRESS);

        // Relay as destination
        const dest: [number, any[]] = [1, []];
        const assetAddressInfo = [
          [PRECOMPILE_NATIVE_ERC20_ADDRESS, amountToSend],
          [await foreignRelayAssetContract.getAddress(), amountToSend],
        ];

        // LocalReserve
        const feesTransferType = 1;

        // DestinationReserve
        const assetsTransferType = 2;

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
              assetsTransferType,
              0n,
              feesTransferType,
              xcmOnDest.toHex(),
            ],
            functionName: "transferAssetsUsingTypeAndThenAddress",
          }),
          gasLimit: 500_000n,
        });

        const result = await context.createBlock(rawTxn);
        expectEVMResult(result.result!.events, "Succeed");

        const assetBalanceAfter = await foreignRelayAssetContract.balanceOf(ALITH_ADDRESS);
        expect(assetBalanceAfter).to.equal(assetBalanceBefore - amountToSend);
      },
    });
  },
});

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
import type { u128 } from "@polkadot/types-codec";
import type { PalletAssetsAssetAccount, PalletAssetsAssetDetails } from "@polkadot/types/lookup";
import { checksumAddress, encodeFunctionData } from "viem";
import {
  expectEVMResult,
  mockOldAssetBalance,
  PARA_1000_SOURCE_LOCATION,
  registerOldForeignAsset,
  mockHrmpChannelExistanceTx,
} from "../../../../helpers";

import type { AssetMetadata } from "../../../../helpers";

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
    let relayAssetId: u128;
    let para1000AssetId: u128;
    let ADDRESS_PARA_1000_ERC20: string;
    const destinationPara = 1000;
    const ADDRESS_RELAY_ERC20 = "0xFfFFfFff1FcaCBd218EDc0EbA20Fc2308C778080";
    const RELAY_ASSET_ID = 42259045809535163221576417993425387648n;
    const amountToSend = 100n;

    beforeAll(async () => {
      const balance = 200000000000000n;
      const assetBalance: PalletAssetsAssetAccount = context
        .polkadotJs()
        .createType("PalletAssetsAssetAccount", {
          balance: balance,
        });
      relayAssetId = context.polkadotJs().createType("u128", RELAY_ASSET_ID);

      const assetDetails: PalletAssetsAssetDetails = context
        .polkadotJs()
        .createType("PalletAssetsAssetDetails", {
          supply: balance,
        });

      await mockOldAssetBalance(
        context,
        assetBalance,
        assetDetails,
        alith,
        relayAssetId,
        ALITH_ADDRESS,
        true
      );

      const { registeredAssetId } = await registerOldForeignAsset(
        context,
        PARA_1000_SOURCE_LOCATION,
        para1000AssetMetadata as any,
        1
      );

      para1000AssetId = context.polkadotJs().createType("u128", registeredAssetId);

      await mockOldAssetBalance(
        context,
        assetBalance,
        assetDetails,
        alith,
        para1000AssetId,
        ALITH_ADDRESS,
        true
      );

      ADDRESS_PARA_1000_ERC20 = "0xFfFFfFff" + para1000AssetId.toHex().slice(2);
      ADDRESS_PARA_1000_ERC20 = checksumAddress(ADDRESS_PARA_1000_ERC20 as `0x${string}`);
    });

    it({
      id: "T01",
      title: "transferAssetsLocation: allows to pay fees with native asset",
      test: async function () {
        const { abi: xcmInterface } = fetchCompiledContract("XCM");
        const assetBalanceBefore = (
          await context.polkadotJs().query.assets.account(relayAssetId.toU8a(), ALITH_ADDRESS)
        )
          .unwrap()
          .balance.toBigInt();

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

        const assetBalanceAfter = (
          await context.polkadotJs().query.assets.account(relayAssetId.toU8a(), ALITH_ADDRESS)
        )
          .unwrap()
          .balance.toBigInt();
        expect(assetBalanceAfter).to.equal(assetBalanceBefore - amountToSend);
      },
    });

    it({
      id: "T02",
      title: "transferAssetsToPara20: allows to pay fees with native asset",
      test: async function () {
        const { abi: xcmInterface } = fetchCompiledContract("XCM");
        const assetBalanceBefore = (
          await context.polkadotJs().query.assets.account(para1000AssetId.toU8a(), ALITH_ADDRESS)
        )
          .unwrap()
          .balance.toBigInt();

        const paraId = destinationPara;

        // Assets must be sorted, so we put the native one first as it has a lower "parents" field.
        const assetAddressInfo = [
          [PRECOMPILE_NATIVE_ERC20_ADDRESS, amountToSend],
          [ADDRESS_PARA_1000_ERC20, amountToSend],
        ];

        const mockHrmp1000Tx = context
          .polkadotJs()
          .tx.sudo.sudo(mockHrmpChannelExistanceTx(context, destinationPara, 1000, 102400, 102400));

        // Change the sudo key so that we avoid nonce issues.
        const sudoKeyTx = context.polkadotJs().tx.sudo.setKey(baltathar.address);
        await context.createBlock(await sudoKeyTx.signAsync(alith), { allowFailures: false });

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

        const assetBalanceAfter = (
          await context.polkadotJs().query.assets.account(para1000AssetId.toU8a(), ALITH_ADDRESS)
        )
          .unwrap()
          .balance.toBigInt();
        expect(assetBalanceAfter).to.equal(assetBalanceBefore - amountToSend);
      },
    });

    it({
      id: "T03",
      title: "transferAssetsToPara32: allows to pay fees with native asset",
      test: async function () {
        const { abi: xcmInterface } = fetchCompiledContract("XCM");
        const assetBalanceBefore = (
          await context.polkadotJs().query.assets.account(para1000AssetId.toU8a(), ALITH_ADDRESS)
        )
          .unwrap()
          .balance.toBigInt();

        const paraId = destinationPara;

        // Assets must be sorted, so we put the native one first as it has a lower "parents" field.
        const assetAddressInfo = [
          [PRECOMPILE_NATIVE_ERC20_ADDRESS, amountToSend],
          [ADDRESS_PARA_1000_ERC20, amountToSend],
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

        const assetBalanceAfter = (
          await context.polkadotJs().query.assets.account(para1000AssetId.toU8a(), ALITH_ADDRESS)
        )
          .unwrap()
          .balance.toBigInt();
        expect(assetBalanceAfter).to.equal(assetBalanceBefore - amountToSend);
      },
    });

    it({
      id: "T04",
      title: "transferAssetsToRelay: allows to pay fees with native asset",
      test: async function () {
        const { abi: xcmInterface } = fetchCompiledContract("XCM");
        const assetBalanceBefore = (
          await context.polkadotJs().query.assets.account(relayAssetId.toU8a(), ALITH_ADDRESS)
        )
          .unwrap()
          .balance.toBigInt();

        // Assets must be sorted, so we put the native one first as it has a lower "parents" field.
        const assetAddressInfo = [
          [PRECOMPILE_NATIVE_ERC20_ADDRESS, amountToSend],
          [ADDRESS_RELAY_ERC20, amountToSend],
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

        const assetBalanceAfter = (
          await context.polkadotJs().query.assets.account(relayAssetId.toU8a(), ALITH_ADDRESS)
        )
          .unwrap()
          .balance.toBigInt();
        expect(assetBalanceAfter).to.equal(assetBalanceBefore - amountToSend);
      },
    });

    it({
      id: "T05",
      title:
        "transferAssetsUsingTypeAndThenLocation (8425d893): allows to pay fees with native asset",
      test: async function () {
        const { abi: xcmInterface } = fetchCompiledContract("XCM");
        const assetBalanceBefore = (
          await context.polkadotJs().query.assets.account(relayAssetId.toU8a(), ALITH_ADDRESS)
        )
          .unwrap()
          .balance.toBigInt();

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

        const assetBalanceAfter = (
          await context.polkadotJs().query.assets.account(relayAssetId.toU8a(), ALITH_ADDRESS)
        )
          .unwrap()
          .balance.toBigInt();
        expect(assetBalanceAfter).to.equal(assetBalanceBefore - amountToSend);
      },
    });

    it({
      id: "T06",
      title:
        "transferAssetsUsingTypeAndThenAddress (8425d893): allows to pay fees with native asset",
      test: async function () {
        const { abi: xcmInterface } = fetchCompiledContract("XCM");
        const assetBalanceBefore = (
          await context.polkadotJs().query.assets.account(relayAssetId.toU8a(), ALITH_ADDRESS)
        )
          .unwrap()
          .balance.toBigInt();

        // Relay as destination
        const dest: [number, any[]] = [1, []];
        const assetAddressInfo = [
          [PRECOMPILE_NATIVE_ERC20_ADDRESS, amountToSend],
          [ADDRESS_RELAY_ERC20, amountToSend],
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

        const assetBalanceAfter = (
          await context.polkadotJs().query.assets.account(relayAssetId.toU8a(), ALITH_ADDRESS)
        )
          .unwrap()
          .balance.toBigInt();
        expect(assetBalanceAfter).to.equal(assetBalanceBefore - amountToSend);
      },
    });
  },
});

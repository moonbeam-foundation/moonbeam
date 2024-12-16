import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, fetchCompiledContract, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, BALTATHAR_ADDRESS, alith, createEthersTransaction } from "@moonwall/util";
import type { u128 } from "@polkadot/types-codec";
import { numberToHex } from "@polkadot/util";
import type { PalletAssetsAssetAccount, PalletAssetsAssetDetails } from "@polkadot/types/lookup";
import { encodeFunctionData } from "viem";
import { expectEVMResult, mockOldAssetBalance } from "../../../../helpers";

const PRECOMPILE_PALLET_XCM_ADDRESS: `0x${string}` = "0x000000000000000000000000000000000000081A";

describeSuite({
  id: "D012900",
  title: "Precompiles - PalletXcm",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let assetId: u128;
    const ADDRESS_ERC20 = "0xFfFFfFff1FcaCBd218EDc0EbA20Fc2308C778080";
    const ASSET_ID = 42259045809535163221576417993425387648n;
    const amountToSend = 100n;

    beforeAll(async () => {
      const balance = 200000000000000n;
      const assetBalance: PalletAssetsAssetAccount = context
        .polkadotJs()
        .createType("PalletAssetsAssetAccount", {
          balance: balance,
        });
      assetId = context.polkadotJs().createType("u128", ASSET_ID);

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
        assetId,
        ALITH_ADDRESS,
        true
      );
    });

    it({
      id: "T01",
      title: "allows to call transferAssetsLocation function",
      test: async function () {
        const { abi: xcmInterface } = fetchCompiledContract("XCM");
        const assetBalanceBefore = (
          await context.polkadotJs().query.assets.account(assetId.toU8a(), ALITH_ADDRESS)
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

        const assetBalanceAfter = (
          await context.polkadotJs().query.assets.account(assetId.toU8a(), ALITH_ADDRESS)
        )
          .unwrap()
          .balance.toBigInt();
        expect(assetBalanceAfter).to.equal(assetBalanceBefore - amountToSend);
      },
    });

    it({
      id: "T02",
      title: "allows to call transferAssetsToPara20 function",
      test: async function () {
        const { abi: xcmInterface } = fetchCompiledContract("XCM");
        const assetBalanceBefore = (
          await context.polkadotJs().query.assets.account(assetId.toU8a(), ALITH_ADDRESS)
        )
          .unwrap()
          .balance.toBigInt();

        const paraId = 1000n;
        const assetAddressInfo = [[ADDRESS_ERC20, amountToSend]];

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

        const assetBalanceAfter = (
          await context.polkadotJs().query.assets.account(assetId.toU8a(), ALITH_ADDRESS)
        )
          .unwrap()
          .balance.toBigInt();
        expect(assetBalanceAfter).to.equal(assetBalanceBefore - amountToSend);
      },
    });

    it({
      id: "T03",
      title: "allows to call transferAssetsToPara32 function",
      test: async function () {
        const { abi: xcmInterface } = fetchCompiledContract("XCM");
        const assetBalanceBefore = (
          await context.polkadotJs().query.assets.account(assetId.toU8a(), ALITH_ADDRESS)
        )
          .unwrap()
          .balance.toBigInt();

        const paraId = 1000n;
        const assetAddressInfo = [[ADDRESS_ERC20, amountToSend]];
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

        const assetBalanceAfter = (
          await context.polkadotJs().query.assets.account(assetId.toU8a(), ALITH_ADDRESS)
        )
          .unwrap()
          .balance.toBigInt();
        expect(assetBalanceAfter).to.equal(assetBalanceBefore - amountToSend);
      },
    });

    it({
      id: "T04",
      title: "allows to call transferAssetsToRelay function",
      test: async function () {
        const { abi: xcmInterface } = fetchCompiledContract("XCM");
        const assetBalanceBefore = (
          await context.polkadotJs().query.assets.account(assetId.toU8a(), ALITH_ADDRESS)
        )
          .unwrap()
          .balance.toBigInt();

        const assetAddressInfo = [[ADDRESS_ERC20, amountToSend]];
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

        const assetBalanceAfter = (
          await context.polkadotJs().query.assets.account(assetId.toU8a(), ALITH_ADDRESS)
        )
          .unwrap()
          .balance.toBigInt();
        expect(assetBalanceAfter).to.equal(assetBalanceBefore - amountToSend);
      },
    });

    it({
      id: "T05",
      title: "allows to call transferAssetsUsingTypeAndThenLocation::8425d893 selector",
      test: async function () {
        const { abi: xcmInterface } = fetchCompiledContract("XCM");
        const assetBalanceBefore = (
          await context.polkadotJs().query.assets.account(assetId.toU8a(), ALITH_ADDRESS)
        )
          .unwrap()
          .balance.toBigInt();

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

        const assetBalanceAfter = (
          await context.polkadotJs().query.assets.account(assetId.toU8a(), ALITH_ADDRESS)
        )
          .unwrap()
          .balance.toBigInt();
        expect(assetBalanceAfter).to.equal(assetBalanceBefore - amountToSend);
      },
    });

    it({
      id: "T06",
      title: "allows to call transferAssetsUsingTypeAndThenLocation::fc19376c selector",
      test: async function () {
        const { abi: xcmInterface } = fetchCompiledContract("XCM");
        const assetBalanceBefore = (
          await context.polkadotJs().query.assets.account(assetId.toU8a(), ALITH_ADDRESS)
        )
          .unwrap()
          .balance.toBigInt();

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

        const assetBalanceAfter = (
          await context.polkadotJs().query.assets.account(assetId.toU8a(), ALITH_ADDRESS)
        )
          .unwrap()
          .balance.toBigInt();
        expect(assetBalanceAfter).to.equal(assetBalanceBefore - amountToSend);
      },
    });

    it({
      id: "T07",
      title: "allows to call transferAssetsUsingTypeAndThenAddress::998093ee selector",
      test: async function () {
        const { abi: xcmInterface } = fetchCompiledContract("XCM");
        const assetBalanceBefore = (
          await context.polkadotJs().query.assets.account(assetId.toU8a(), ALITH_ADDRESS)
        )
          .unwrap()
          .balance.toBigInt();

        // Relay as destination
        const dest: [number, any[]] = [1, []];
        const assetAddressInfo = [[ADDRESS_ERC20, amountToSend]];

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

        const assetBalanceAfter = (
          await context.polkadotJs().query.assets.account(assetId.toU8a(), ALITH_ADDRESS)
        )
          .unwrap()
          .balance.toBigInt();
        expect(assetBalanceAfter).to.equal(assetBalanceBefore - amountToSend);
      },
    });

    it({
      id: "T08",
      title: "allows to call transferAssetsUsingTypeAndThenAddress::aaecfc62 selector",
      test: async function () {
        const { abi: xcmInterface } = fetchCompiledContract("XCM");
        const assetBalanceBefore = (
          await context.polkadotJs().query.assets.account(assetId.toU8a(), ALITH_ADDRESS)
        )
          .unwrap()
          .balance.toBigInt();

        const paraIdInHex = numberToHex(2000, 32);
        const parachain_enum_selector = "0x00";

        // This represents X2(Parent, Parachain(2000))
        const dest: [number, any[]] = [1, [parachain_enum_selector + paraIdInHex.slice(2)]];
        const assetAddressInfo = [[ADDRESS_ERC20, amountToSend]];
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

        const assetBalanceAfter = (
          await context.polkadotJs().query.assets.account(assetId.toU8a(), ALITH_ADDRESS)
        )
          .unwrap()
          .balance.toBigInt();
        expect(assetBalanceAfter).to.equal(assetBalanceBefore - amountToSend);
      },
    });
  },
});

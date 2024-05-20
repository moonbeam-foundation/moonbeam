import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, fetchCompiledContract, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, BALTATHAR_ADDRESS, alith, createEthersTransaction } from "@moonwall/util";
import { u128 } from "@polkadot/types-codec";
import { PalletAssetsAssetAccount, PalletAssetsAssetDetails } from "@polkadot/types/lookup";
import { encodeFunctionData } from "viem";
import { expectEVMResult, mockAssetBalance } from "../../../../helpers";

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
    const weight = { refTime: 5000000000, proofSize: 40000 };

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

      await mockAssetBalance(
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
            args: [dest, beneficiary, assetLocationInfo, 0, weight],
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
            args: [paraId, BALTATHAR_ADDRESS, assetAddressInfo, 0, weight],
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
            args: [paraId, beneficiaryAddress, assetAddressInfo, 0, weight],
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
            args: [beneficiaryAddress, assetAddressInfo, 0, weight],
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
  },
});

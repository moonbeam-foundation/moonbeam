import "@moonbeam-network/api-augment";
import { beforeAll, deployCreateCompiledContract, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, BALTATHAR_ADDRESS, alith, createViemTransaction } from "@moonwall/util";
import type { u128 } from "@polkadot/types-codec";
import type { PalletAssetsAssetAccount, PalletAssetsAssetDetails } from "@polkadot/types/lookup";
import { type Abi, encodeFunctionData } from "viem";
import { mockOldAssetBalance } from "../../../../helpers";

describeSuite({
  id: "D022806",
  title: "Precompiles - Assets-ERC20 Wasm",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let assetId: u128;
    let erc20Abi: Abi;
    const ASSET_ID = 42259045809535163221576417993425387648n;
    const ADDRESS_ERC20 = "0xFfFFfFff1FcaCBd218EDc0EbA20Fc2308C778080" as `0x${string}`;

    beforeAll(async () => {
      // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
      // And we need relay tokens for issuing a transaction to be executed in the relay
      const balance = context.polkadotJs().createType("Balance", 100000000000000);
      const assetBalance: PalletAssetsAssetAccount = context
        .polkadotJs()
        .createType("PalletAssetsAssetAccount", {
          balance: balance,
        });

      const assetDetails: PalletAssetsAssetDetails = context
        .polkadotJs()
        .createType("PalletAssetsAssetDetails", {
          supply: balance,
        });
      assetId = context.polkadotJs().createType("u128", ASSET_ID);

      await mockOldAssetBalance(
        context,
        assetBalance,
        assetDetails,
        alith,
        assetId,
        ALITH_ADDRESS,
        true
      );

      const { abi } = await deployCreateCompiledContract(context, "ERC20Instance");
      erc20Abi = abi;
    });

    it({
      id: "T01",
      title: "allows to transfer",
      test: async function () {
        const { result } = await context.createBlock(
          createViemTransaction(context, {
            to: ADDRESS_ERC20,
            data: encodeFunctionData({
              functionName: "transfer",
              args: [BALTATHAR_ADDRESS, 1000],
              abi: erc20Abi,
            }),
          })
        );

        // const receipt = await context.web3.eth.getTransactionReceipt(result.hash);
        const receipt = await context
          .viem()
          .getTransactionReceipt({ hash: result?.hash as `0x${string}` });
        expect(receipt.status).to.equal("success");

        // Baltathar balance is 1000
        const baltatharBalance = await context
          .polkadotJs()
          .query.assets.account(assetId.toU8a(), BALTATHAR_ADDRESS);
        expect(baltatharBalance.unwrap().balance.toBigInt()).to.equal(1000n);
      },
    });
  },
});

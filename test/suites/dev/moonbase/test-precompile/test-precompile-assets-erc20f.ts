import "@moonbeam-network/api-augment";
import { beforeAll, deployCreateCompiledContract, describeSuite, expect } from "@moonwall/cli";
import { BALTATHAR_ADDRESS, alith, createViemTransaction } from "@moonwall/util";
import type { u128 } from "@polkadot/types-codec";
import type { PalletAssetsAssetAccount, PalletAssetsAssetDetails } from "@polkadot/types/lookup";
import { type Abi, encodeFunctionData } from "viem";
import { mockOldAssetBalance } from "../../../../helpers";

describeSuite({
  id: "D012809",
  title: "Precompiles - Assets-ERC20 Wasm",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let assetId: u128;
    let erc20Abi: Abi;
    let erc20InstanceAddress: `0x${string}`;
    const ASSET_ID = 42259045809535163221576417993425387648n;

    beforeAll(async () => {
      // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
      // And we need relay tokens for issuing a transaction to be executed in the relay
      const balance = context.polkadotJs().createType("Balance", 100000000000000);
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

      const { abi, contractAddress } = await deployCreateCompiledContract(context, "ERC20Instance");
      erc20Abi = abi;
      erc20InstanceAddress = contractAddress;

      await mockOldAssetBalance(
        context,
        assetBalance,
        assetDetails,
        alith,
        assetId,
        erc20InstanceAddress
      );
    });

    it({
      id: "T01",
      title: "allows to transfer through call from SC ",
      test: async function () {
        // Create approval
        const { result } = await context.createBlock(
          createViemTransaction(context, {
            to: erc20InstanceAddress,
            data: encodeFunctionData({
              abi: erc20Abi,
              functionName: "transfer",
              args: [BALTATHAR_ADDRESS, 1000],
            }),
          })
        );

        const receipt = await context
          .viem("public")
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

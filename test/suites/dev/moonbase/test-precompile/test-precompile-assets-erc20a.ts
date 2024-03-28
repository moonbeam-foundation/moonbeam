import "@moonbeam-network/api-augment";
import { beforeAll, deployCreateCompiledContract, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, BALTATHAR_ADDRESS, alith, createEthersTransaction } from "@moonwall/util";
import { u128 } from "@polkadot/types-codec";
import { PalletAssetsAssetAccount, PalletAssetsAssetDetails } from "@polkadot/types/lookup";
import { mockAssetBalance } from "../../../../helpers";

import { Abi, encodeFunctionData } from "viem";

describeSuite({
  id: "D012804",
  title: "",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let assetId: u128;
    let erc20Abi: Abi;
    const ASSET_ID = 42259045809535163221576417993425387648n;
    const ADDRESS_ERC20 = "0xFfFFfFff1FcaCBd218EDc0EbA20Fc2308C778080" as `0x${string}`;
    const SELECTORS = {
      balanceOf: "70a08231",
      totalSupply: "18160ddd",
      approve: "095ea7b3",
      allowance: "dd62ed3e",
      transfer: "a9059cbb",
      transferFrom: "23b872dd",
      logApprove: "0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925",
      logTransfer: "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef",
    };

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

      await mockAssetBalance(
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
      title: "allows to approve transfers, and allowance matches",
      test: async function () {
        const rawSigned = await createEthersTransaction(context, {
          to: ADDRESS_ERC20,
          data: encodeFunctionData({
            abi: erc20Abi,
            functionName: "approve",
            args: [BALTATHAR_ADDRESS, 1000],
          }),
        });

        const { result } = await context.createBlock(rawSigned);
        const receipt = await context
          .viem("public")
          .getTransactionReceipt({ hash: result?.hash as `0x${string}` });

        expect(receipt.status).to.equal("success");
        expect(receipt.logs.length).to.eq(1);
        expect(receipt.logs[0].address.toLowerCase()).to.eq(ADDRESS_ERC20.toLowerCase());
        expect(receipt.logs[0].topics.length).to.eq(3);
        expect(receipt.logs[0].topics[0]).to.eq(SELECTORS.logApprove);
        const approvals = await context
          .polkadotJs()
          .query.assets.approvals(assetId.toU8a(), ALITH_ADDRESS, BALTATHAR_ADDRESS);
        expect(approvals.unwrap().amount.toBigInt()).to.equal(1000n);
      },
    });

    it({
      id: "T02",
      title: "should gather the allowance",
      test: async function () {
        const data = await context.viem().readContract({
          address: ADDRESS_ERC20,
          abi: erc20Abi,
          functionName: "allowance",
          args: [ALITH_ADDRESS, BALTATHAR_ADDRESS],
        });
        expect(data).toBe(1000n);
      },
    });
  },
});

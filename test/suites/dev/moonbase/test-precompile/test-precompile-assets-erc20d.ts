import "@moonbeam-network/api-augment";
import { beforeAll, deployCreateCompiledContract, describeSuite, expect } from "@moonwall/cli";
import {
  BALTATHAR_ADDRESS,
  BALTATHAR_PRIVATE_KEY,
  CHARLETH_ADDRESS,
  alith,
  createViemTransaction,
} from "@moonwall/util";
import type { u128 } from "@polkadot/types-codec";
import type { PalletAssetsAssetAccount, PalletAssetsAssetDetails } from "@polkadot/types/lookup";
import { type Abi, encodeFunctionData } from "viem";
import { mockOldAssetBalance } from "../../../../helpers";

describeSuite({
  id: "D012807",
  title: "Precompiles - Assets-ERC20 Wasm",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let assetId: u128;
    let erc20Abi: Abi;
    let erc20InstanceAddress: `0x${string}`;
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

      assetId = context.polkadotJs().createType("u128", ASSET_ID);
      const assetDetails: PalletAssetsAssetDetails = context
        .polkadotJs()
        .createType("PalletAssetsAssetDetails", {
          supply: balance,
        });

      const { abi, contractAddress } = await deployCreateCompiledContract(context, "ERC20Instance");
      erc20Abi = abi;
      erc20InstanceAddress = contractAddress;
      // We fund the contract address with this test
      await mockOldAssetBalance(
        context,
        assetBalance,
        assetDetails,
        alith,
        assetId,
        erc20InstanceAddress,
        true
      );
    });

    it({
      id: "T01",
      title: "allows to approve transfer and use transferFrom from contract calls",
      test: async function () {
        // Create approval
        const blockAlith = await context.createBlock(
          createViemTransaction(context, {
            to: erc20InstanceAddress,
            data: encodeFunctionData({
              functionName: "approve",
              abi: erc20Abi,
              args: [BALTATHAR_ADDRESS, 1000],
            }),
          })
        );

        const receiptAlith = await context
          .viem("public")
          .getTransactionReceipt({ hash: blockAlith.result?.hash as `0x${string}` });

        expect(receiptAlith.status).to.equal("success");
        expect(receiptAlith.logs.length).to.eq(1);
        expect(receiptAlith.logs[0].address.toLowerCase()).to.eq(ADDRESS_ERC20.toLowerCase());
        expect(receiptAlith.logs[0].topics.length).to.eq(3);
        expect(receiptAlith.logs[0].topics[0]).to.eq(SELECTORS.logApprove);

        const approvals = await context
          .polkadotJs()
          .query.assets.approvals(assetId.toU8a(), erc20InstanceAddress, BALTATHAR_ADDRESS);

        expect(approvals.unwrap().amount.toBigInt()).to.equal(1000n);
        // We are gonna spend 1000 from contractInstanceAddress to send it to charleth
        // Since this is a regular call, it will take contractInstanceAddress as msg.sender
        // thus from & to will be the same, and approval wont be touched
        const blockBaltathar = await context.createBlock(
          createViemTransaction(context, {
            privateKey: BALTATHAR_PRIVATE_KEY,
            to: erc20InstanceAddress,
            data: encodeFunctionData({
              abi: erc20Abi,
              functionName: "transferFrom",
              args: [erc20InstanceAddress, CHARLETH_ADDRESS, 1000],
            }),
          })
        );

        const receiptBaltathar = await context
          .viem("public")
          .getTransactionReceipt({ hash: blockBaltathar.result?.hash as `0x${string}` });

        expect(receiptBaltathar.logs.length).to.eq(1);
        expect(receiptBaltathar.logs[0].address.toLowerCase()).to.eq(ADDRESS_ERC20.toLowerCase());
        expect(receiptBaltathar.logs[0].topics.length).to.eq(3);
        expect(receiptBaltathar.logs[0].topics[0]).to.eq(SELECTORS.logTransfer);
        expect(receiptBaltathar.status).to.equal("success");

        // approvals are untouched
        const newApprovals = await context
          .polkadotJs()
          .query.assets.approvals(assetId.toU8a(), erc20InstanceAddress, BALTATHAR_ADDRESS);
        expect(newApprovals.unwrap().amount.toBigInt()).to.equal(1000n);

        const estimatedGas = await context.viem().estimateGas({
          account: BALTATHAR_ADDRESS,
          to: ADDRESS_ERC20,
          data: encodeFunctionData({
            abi: erc20Abi,
            functionName: "transferFrom",
            args: [erc20InstanceAddress, CHARLETH_ADDRESS, 1000],
          }),
        });

        // Snapshot estimated gas
        expect(estimatedGas).toMatchInlineSnapshot(`55270n`);

        // this time we call directly from Baltathar the ERC20 contract
        const directBlock = await context.createBlock(
          createViemTransaction(context, {
            privateKey: BALTATHAR_PRIVATE_KEY,
            gas: estimatedGas,
            to: ADDRESS_ERC20,
            data: encodeFunctionData({
              functionName: "transferFrom",
              abi: erc20Abi,
              args: [erc20InstanceAddress, CHARLETH_ADDRESS, 1000],
            }),
          })
        );

        const directReceipt = await context
          .viem("public")
          .getTransactionReceipt({ hash: directBlock.result?.hash as `0x${string}` });

        expect(directReceipt.logs.length).to.eq(1);
        expect(directReceipt.logs[0].address.toLowerCase()).to.eq(ADDRESS_ERC20.toLowerCase());
        expect(directReceipt.logs[0].topics.length).to.eq(3);
        expect(directReceipt.logs[0].topics[0]).to.eq(SELECTORS.logTransfer);
        expect(directReceipt.status).to.equal("success");

        // Approve amount is null now
        const directApprovals = await context
          .polkadotJs()
          .query.assets.approvals(assetId.toU8a(), erc20InstanceAddress, BALTATHAR_ADDRESS);
        expect(directApprovals.isNone).to.eq(true);

        // Charleth balance is 2000
        const charletBalance = await context
          .polkadotJs()
          .query.assets.account(assetId.toU8a(), CHARLETH_ADDRESS);
        expect(charletBalance.unwrap().balance.toBigInt()).to.equal(2000n);
      },
    });
  },
});

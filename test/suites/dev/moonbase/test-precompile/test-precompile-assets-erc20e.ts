import "@moonbeam-network/api-augment";
import { beforeAll, deployCreateCompiledContract, describeSuite, expect } from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  BALTATHAR_PRIVATE_KEY,
  CHARLETH_ADDRESS,
  alith,
  createViemTransaction,
} from "@moonwall/util";
import { u128 } from "@polkadot/types-codec";
import { PalletAssetsAssetAccount, PalletAssetsAssetDetails } from "@polkadot/types/lookup";
import { Abi, encodeFunctionData } from "viem";
import { mockOldAssetBalance } from "../../../../helpers";

describeSuite({
  id: "D012808",
  title: "Precompiles - Assets-ERC20 Wasm",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let assetId: u128;
    let erc20Abi: Abi;
    let erc20InstanceAddress: `0x${string}`;
    const ASSET_ID = 42259045809535163221576417993425387648n;
    const ADDRESS_ERC20 = "0xffffffff1fcacbd218edc0eba20fc2308c778080" as `0x${string}`;
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

      const { contractAddress, abi } = await deployCreateCompiledContract(context, "ERC20Instance");
      erc20InstanceAddress = contractAddress;
      erc20Abi = abi;

      // We fund Alith with this test
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
      title: "Bob approves contract and use transferFrom from contract calls",
      test: async function () {
        const tx = await createViemTransaction(context, {
          to: ADDRESS_ERC20,
          data: encodeFunctionData({
            functionName: "approve",
            args: [erc20InstanceAddress, 1000],
            abi: erc20Abi,
          }),
        });

        const { result } = await context.createBlock(tx);
        const receipt = await context
          .viem("public")
          .getTransactionReceipt({ hash: result?.hash as `0x${string}` });

        expect(receipt.status).to.equal("success");
        expect(receipt.logs.length).to.eq(1);
        expect(receipt.logs[0].address).to.eq(ADDRESS_ERC20);
        expect(receipt.logs[0].topics.length).to.eq(3);
        expect(receipt.logs[0].topics[0]).to.eq(SELECTORS.logApprove);

        const approvals = await context
          .polkadotJs()
          .query.assets.approvals(assetId.toU8a(), ALITH_ADDRESS, erc20InstanceAddress);

        expect(approvals.unwrap().amount.toBigInt()).to.equal(1000n);
        // We are gonna spend 1000 from ALITH_ADDRESS to send it to charleth from contract address
        // even if Bob calls, msg.sender will become the contract with regular calls
        const blockBaltathar = await context.createBlock(
          createViemTransaction(context, {
            privateKey: BALTATHAR_PRIVATE_KEY,
            to: erc20InstanceAddress,
            data: encodeFunctionData({
              functionName: "transferFrom",
              args: [ALITH_ADDRESS, CHARLETH_ADDRESS, 1000],
              abi: erc20Abi,
            }),
          })
        );

        const receiptBaltathar = await context
          .viem("public")
          .getTransactionReceipt({ hash: blockBaltathar.result?.hash as `0x${string}` });
        expect(receiptBaltathar.logs.length).to.eq(1);
        expect(receiptBaltathar.logs[0].address).to.eq(ADDRESS_ERC20);
        expect(receiptBaltathar.logs[0].topics.length).to.eq(3);
        expect(receiptBaltathar.logs[0].topics[0]).to.eq(SELECTORS.logTransfer);
        expect(receiptBaltathar.status).to.equal("success");

        // Approve amount is null now
        const approvalBaltathar = await context
          .polkadotJs()
          .query.assets.approvals(assetId.toU8a(), ALITH_ADDRESS, erc20InstanceAddress);
        expect(approvalBaltathar.isNone).to.eq(true);

        // Charleth balance is 1000
        const charletBalance = await context
          .polkadotJs()
          .query.assets.account(assetId.toU8a(), CHARLETH_ADDRESS);
        expect(charletBalance.unwrap().balance.toBigInt()).to.equal(1000n);
      },
    });
  },
});

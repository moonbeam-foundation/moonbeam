import "@moonbeam-network/api-augment";
import {
  TransactionTypes,
  beforeAll,
  deployCreateCompiledContract,
  describeSuite,
  expect,
} from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  BALTATHAR_ADDRESS,
  CHARLETH_ADDRESS,
  alith,
  createEthersTransaction,
} from "@moonwall/util";
import type { u128 } from "@polkadot/types";
import type { PalletAssetsAssetAccount, PalletAssetsAssetDetails } from "@polkadot/types/lookup";
import { nToHex } from "@polkadot/util";
import { type Abi, encodeFunctionData } from "viem";
import { mockOldAssetBalance } from "../../../../helpers";

describeSuite({
  id: "D022802",
  title: "Precompiles - Low Level Transactions",
  foundationMethods: "dev",
  testCases: ({ context, log, it }) => {
    let assetId: u128;
    let contractInstanceAddress: `0x${string}`;
    let contractAbi: Abi;

    const ASSET_ID = 42259045809535163221576417993425387648n;
    const MAX_SUPPLY = 100000000000000;

    beforeAll(async function () {
      // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
      // And we need relay tokens for issuing a transaction to be executed in the relay
      const balance = context.polkadotJs().createType("Balance", MAX_SUPPLY);
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
      contractInstanceAddress = contractAddress;
      contractAbi = abi;

      await mockOldAssetBalance(
        context,
        assetBalance,
        assetDetails,
        alith,
        assetId,
        contractInstanceAddress,
        true
      );

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

    let testCounter = 2;

    it({
      id: "T01",
      title: "can make static calls to view functions",
      test: async function () {
        const callResult = await context.viem().call({
          account: ALITH_ADDRESS,
          to: contractInstanceAddress,
          data: encodeFunctionData({
            abi: contractAbi,
            functionName: "totalSupply_static",
          }),
        });

        expect(callResult.data).equals(nToHex(MAX_SUPPLY, { bitLength: 256 }));
      },
    });

    for (const txnType of TransactionTypes) {
      it({
        id: `T${testCounter < 10 ? "0" : ""}${testCounter++}`,
        title: `can make static calls to view functions and transact ${txnType}`,
        test: async function () {
          await context.createBlock(
            await createEthersTransaction(context, {
              to: contractInstanceAddress,
              data: encodeFunctionData({
                abi: contractAbi,
                functionName: "approve_max_supply",
                args: [CHARLETH_ADDRESS],
              }),
              txnType: "eip1559",
            })
          );

          const approvals = await context
            .polkadotJs()
            .query.assets.approvals(assetId.toU8a(), contractInstanceAddress, CHARLETH_ADDRESS);

          expect(approvals.unwrap().amount.toNumber()).to.equal(MAX_SUPPLY);
        },
      });

      it({
        id: `T${testCounter < 10 ? "0" : ""}${testCounter++}`,
        title: `has unchanged state when submitting static call ${txnType}`,
        test: async function () {
          const { result } = await context.createBlock(
            await createEthersTransaction(context, {
              to: contractInstanceAddress,
              data: encodeFunctionData({
                abi: contractAbi,
                functionName: "approve_static",
                args: [BALTATHAR_ADDRESS, 1000],
              }),
            })
          );

          const approvals = await context
            .polkadotJs()
            .query.assets.approvals(assetId.toU8a(), contractInstanceAddress, BALTATHAR_ADDRESS);

          expect(result?.successful, "Call unsuccessful").to.be.true;
          expect(approvals.isNone).to.be.true;
        },
      });

      it({
        id: `T${testCounter < 10 ? "0" : ""}${testCounter++}`,
        title: `visibility preserved for static calls ${txnType}`,
        test: async function () {
          const { result } = await context.createBlock(
            await createEthersTransaction(context, {
              to: contractInstanceAddress,
              data: encodeFunctionData({
                abi: contractAbi,
                functionName: "approve_ext_static",
                args: [BALTATHAR_ADDRESS, 1000],
              }),
            })
          );

          const approvals = await context
            .polkadotJs()
            .query.assets.approvals(assetId.toU8a(), contractInstanceAddress, BALTATHAR_ADDRESS);

          expect(result?.successful, "Call unsuccessful").to.be.true;
          expect(approvals.isNone).to.be.true;
        },
      });

      it({
        id: `T${testCounter < 10 ? "0" : ""}${testCounter++}`,
        title: `visibility preserved for delegate->static calls ${txnType}`,
        test: async function () {
          const { result } = await context.createBlock(
            await createEthersTransaction(context, {
              to: contractInstanceAddress,
              data: encodeFunctionData({
                abi: contractAbi,
                functionName: "approve_delegate_to_static",
                args: [BALTATHAR_ADDRESS, 1000],
              }),
            })
          );

          const approvals = await context
            .polkadotJs()
            .query.assets.approvals(assetId.toU8a(), contractInstanceAddress, BALTATHAR_ADDRESS);

          expect(result?.successful, "Call unsuccessful").to.be.true;
          expect(approvals.isNone).to.be.true;
        },
      });

      it({
        id: `T${testCounter < 10 ? "0" : ""}${testCounter++}`,
        title: `visibility preserved for static->delegate calls ${txnType}`,
        test: async function () {
          const { result } = await context.createBlock(
            await createEthersTransaction(context, {
              to: contractInstanceAddress,
              data: encodeFunctionData({
                abi: contractAbi,
                functionName: "approve_static_to_delegate",
                args: [BALTATHAR_ADDRESS, 1000],
              }),
            })
          );

          const approvals = await context
            .polkadotJs()
            .query.assets.approvals(assetId.toU8a(), contractInstanceAddress, BALTATHAR_ADDRESS);

          expect(result?.successful, "Call unsuccessful").to.be.true;
          expect(approvals.isNone).to.be.true;
        },
      });
    }
  },
});

import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import type { ApiPromise } from "@polkadot/api";
import { alith, ALITH_ADDRESS, baltathar, ethan, TREASURY_ACCOUNT } from "@moonwall/util";
import { FrameSupportPalletId } from "@polkadot/types/lookup";
import { sendCallAsPara } from "../../../../helpers/xcm";
import { expectSystemEvent } from "../../../../helpers/expect";
import {
  addAssetToWeightTrader,
  assetContractAddress,
  foreignAssetBalance,
  mockAssetBalance,
  PARA_1000_SOURCE_LOCATION,
} from "../../../../helpers";
import { getContract } from "viem";

describeSuite({
  id: "D013803",
  title: "Treasury pallet spend using non-native assets",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let treasuryPalletId: FrameSupportPalletId;
    let treasuryAddress: string;
    let api: ApiPromise;
    let assetKind;
    const assetId = 111n;

    beforeAll(async function () {
      api = context.polkadotJs();
      assetKind = api.createType("FrameSupportTokensFungibleUnionOfNativeOrWithId", assetId);
    });

    it({
      id: "T01",
      title: "Treasury can spend in non-native assets",
      test: async function () {
        const assetLocation = PARA_1000_SOURCE_LOCATION;

        const createForeignAssetCall = context
          .polkadotJs()
          .tx.evmForeignAssets.createForeignAsset(assetId, assetLocation, 18, "TEST", "TEST");

        const sudoCall = context.polkadotJs().tx.sudo.sudo(createForeignAssetCall);
        const block = await context.createBlock(sudoCall, { allowFailures: false });
        await expectSystemEvent(
          block.block.hash,
          "evmForeignAssets",
          "ForeignAssetCreated",
          context
        );

        // Add asset to weight trader
        await addAssetToWeightTrader(assetLocation, 1_000_000_000_000_000_000n, context);

        // Fund the treasury account
        const treasuryBalance = 100_000_000_000_000_000_000n;
        await mockAssetBalance(context, treasuryBalance, assetId, alith, TREASURY_ACCOUNT);
        const newBalance = await foreignAssetBalance(context, assetId, TREASURY_ACCOUNT);
        expect(newBalance).toBe(treasuryBalance);

        // Spend from treasury account
        const proposal_value = 1_000_000_000_000_000_000n;
        const tx = api.tx.treasury.spend(assetKind, proposal_value, ethan.address, null);
        const signedTx = await api.tx.sudo.sudo(tx).signAsync(alith);
        await context.createBlock(signedTx, {
          allowFailures: false,
          expectEvents: [api.events.treasury.AssetSpendApproved],
        });

        // Spending was successfully submitted
        expect((await api.query.treasury.spendCount()).toNumber()).to.equal(1);

        const contractAddress = assetContractAddress(assetId);
        const { abi, bytecode } = fetchCompiledContract("ERC20Instance");

        const totalSupply = await context.viem().readContract({
          address: contractAddress,
          abi,
          functionName: "totalSupply",
        });

        console.log("Total supply: ", totalSupply);

        // Trigger payout
        await context.createBlock(await api.tx.treasury.payout(0).signAsync(ethan), {
          allowFailures: false,
          expectEvents: [api.events.treasury.Paid],
        });
        // const newBalanceAfter = await foreignAssetBalance(context, assetId, TREASURY_ACCOUNT);
        // expect(newBalanceAfter).toBe(treasuryBalance - proposal_value);
      },
    });
  },
});

import "@moonbeam-network/api-augment";
import { alith, baltathar, beforeAll, describeSuite, expect } from "moonwall";

import {
  XcmFragment,
  type RawXcmMessage,
  injectHrmpMessageAndSeal,
  sovereignAccountOfSibling,
  sendCallAsPara,
  foreignAssetBalance,
  addAssetToWeightTrader,
} from "../../../../helpers";
import { fundAccount } from "../../../../helpers/balances.js";
import { expectSubstrateEvent, expectSystemEvent } from "../../../../helpers/expect.js";

describeSuite({
  id: "D020113",
  title: "Freeze -> Deposit -> Unfreeze -> Permissionless Claim",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    const fundAmount = 100_000_000_000_000_000_000_000n;
    const assetId = 8;
    const paraId = 3000;
    const depositAmount = 10_000_000_000_000_000_000n; // 10 tokens (18 decimals)
    const assetLocation = {
      parents: 1,
      interior: {
        X3: [{ Parachain: paraId }, { PalletInstance: 8 }, { GeneralIndex: 8 }],
      },
    };

    beforeAll(async () => {
      // Fund sovereign account
      const sovereignAccount = sovereignAccountOfSibling(context, paraId);
      await fundAccount(sovereignAccount as `0x${string}`, fundAmount, context);

      // Create a foreign asset via XCM
      const createForeignAssetCall = context
        .polkadotJs()
        .tx.evmForeignAssets.createForeignAsset(assetId, assetLocation, 18, "PENDING", "PEND");
      const { blockRes } = await sendCallAsPara(
        createForeignAssetCall,
        paraId,
        context,
        fundAmount / 20n
      );
      await expectSystemEvent(
        blockRes.block.hash,
        "evmForeignAssets",
        "ForeignAssetCreated",
        context
      );

      // Add asset to weight trader so XCM deposits work
      await addAssetToWeightTrader(assetLocation, 0n, context);
    });

    it({
      id: "T01",
      title:
        "Should record pending deposit when frozen with XCM deposits allowed, then allow permissionless claim after unfreeze",
      test: async function () {
        // 1. Freeze asset with allow_xcm_deposit = true
        const freezeCall = context
          .polkadotJs()
          .tx.evmForeignAssets.freezeForeignAsset(assetId, true);
        const { blockRes: freezeBlock } = await sendCallAsPara(
          freezeCall,
          paraId,
          context,
          fundAmount / 20n
        );
        await expectSystemEvent(
          freezeBlock.block.hash,
          "evmForeignAssets",
          "ForeignAssetFrozen",
          context
        );

        // Verify the asset is frozen with XCM deposit allowed
        const assetAfterFreeze = (
          await context.polkadotJs().query.evmForeignAssets.assetsByLocation(assetLocation)
        ).toJSON();
        expect(assetAfterFreeze![1]).to.eq("FrozenXcmDepositAllowed");

        // 2. Send XCM deposit while asset is frozen
        const beneficiary = alith.address;
        const alithBalanceBefore = await foreignAssetBalance(
          context,
          BigInt(assetId),
          beneficiary as `0x${string}`
        );

        const xcmMessage = new XcmFragment({
          assets: [
            {
              multilocation: assetLocation,
              fungible: depositAmount,
            },
          ],
          weight_limit: {
            refTime: 1_000_000_000_000n,
            proofSize: 256 * 1024,
          },
          beneficiary: beneficiary,
        })
          .reserve_asset_deposited()
          .clear_origin()
          .buy_execution()
          .deposit_asset()
          .as_v4();

        await injectHrmpMessageAndSeal(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        // Verify balance did NOT increase (deposit is pending, not minted)
        const alithBalanceAfterDeposit = await foreignAssetBalance(
          context,
          BigInt(assetId),
          beneficiary as `0x${string}`
        );
        expect(alithBalanceAfterDeposit).to.eq(alithBalanceBefore);

        // Verify pending deposit exists in storage
        const pendingDeposit = (
          await context.polkadotJs().query.evmForeignAssets.pendingDeposits(assetId, beneficiary)
        ).toJSON();
        expect(BigInt(pendingDeposit as string)).to.eq(depositAmount);

        // 3. Unfreeze the asset (via XCM from the owning parachain)
        const unfreezeCall = context.polkadotJs().tx.evmForeignAssets.unfreezeForeignAsset(assetId);
        const { blockRes: unfreezeBlock } = await sendCallAsPara(
          unfreezeCall,
          paraId,
          context,
          fundAmount / 20n
        );
        await expectSystemEvent(
          unfreezeBlock.block.hash,
          "evmForeignAssets",
          "ForeignAssetUnfrozen",
          context
        );

        // Verify asset is active again
        const assetAfterUnfreeze = (
          await context.polkadotJs().query.evmForeignAssets.assetsByLocation(assetLocation)
        ).toJSON();
        expect(assetAfterUnfreeze![1]).to.eq("Active");

        // 4. Permissionless claim: baltathar claims the pending deposit for alith
        const claimCall = context
          .polkadotJs()
          .tx.evmForeignAssets.claimPendingDeposit(assetId, beneficiary);
        const claimBlock = await context.createBlock(claimCall.signAsync(baltathar));
        expectSubstrateEvent(claimBlock, "evmForeignAssets", "PendingDepositClaimed");

        // Verify the beneficiary (alith) received the tokens
        const alithBalanceAfterClaim = await foreignAssetBalance(
          context,
          BigInt(assetId),
          beneficiary as `0x${string}`
        );
        expect(alithBalanceAfterClaim - alithBalanceBefore).to.eq(depositAmount);

        // Verify pending deposit was cleared
        const pendingDepositAfter = (
          await context.polkadotJs().query.evmForeignAssets.pendingDeposits(assetId, beneficiary)
        ).toJSON();
        expect(pendingDepositAfter).to.be.null;
      },
    });
  },
});

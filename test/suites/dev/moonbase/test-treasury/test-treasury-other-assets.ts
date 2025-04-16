import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import type { ApiPromise } from "@polkadot/api";
import { alith, ethan, TREASURY_ACCOUNT } from "@moonwall/util";
import {
  foreignAssetBalance,
  PARA_1000_SOURCE_LOCATION,
  registerAndFundAsset,
  type TestAsset,
} from "../../../../helpers";

describeSuite({
  id: "D013803",
  title: "Treasury pallet spend using foreign assets",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let api: ApiPromise;
    let assetKind;
    const initialBalance: bigint = 500_000_000_000_000_000n;
    const xcIntrAsset: TestAsset = {
      id: 1000100010001000n,
      location: PARA_1000_SOURCE_LOCATION,
      metadata: {
        name: "TEST",
        symbol: "TEST",
        decimals: 19n,
        isFrozen: false,
      },
      relativePrice: 1_000_000_000_000_000_000n,
    };

    beforeAll(async function () {
      api = context.polkadotJs();
      assetKind = api.createType("FrameSupportTokensFungibleUnionOfNativeOrWithId", {
        WithId: xcIntrAsset.id,
      });
    });

    it({
      id: "T01",
      title: "Treasury can spend in non-native assets",
      test: async function () {
        // Register foreign asset used to pay fees (i.e. xcINTR)
        await registerAndFundAsset(context, xcIntrAsset, initialBalance, TREASURY_ACCOUNT);

        const newBalance = await foreignAssetBalance(
          context,
          BigInt(xcIntrAsset.id),
          TREASURY_ACCOUNT
        );
        expect(newBalance).toBe(initialBalance);

        // Treasury proposal spend value is half of the balance to ensure reducible balance covers the amount
        const proposal_value = initialBalance / 2n;
        const tx = api.tx.treasury.spend(assetKind, proposal_value, ethan.address, null);
        const signedTx = await api.tx.sudo.sudo(tx).signAsync(alith);
        await context.createBlock(signedTx, {
          allowFailures: false,
          expectEvents: [api.events.treasury.AssetSpendApproved],
        });

        expect((await api.query.treasury.spendCount()).toNumber()).to.equal(1);

        // Trigger payout
        await context.createBlock(await api.tx.treasury.payout(0).signAsync(ethan), {
          allowFailures: false,
          expectEvents: [api.events.treasury.Paid],
        });

        const balance = await foreignAssetBalance(
          context,
          BigInt(xcIntrAsset.id),
          ethan.address as `0x${string}`
        );
        expect(balance).toBe(proposal_value);

        const newBalanceAfter = await foreignAssetBalance(
          context,
          BigInt(xcIntrAsset.id),
          TREASURY_ACCOUNT
        );
        expect(newBalanceAfter).toBe(initialBalance - proposal_value);
      },
    });
  },
});

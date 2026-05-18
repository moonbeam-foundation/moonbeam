import "@moonbeam-network/api-augment/moonbase";
import { TREASURY_ACCOUNT, alith, beforeAll, describeSuite, ethan, expect } from "moonwall";
import type { ApiPromise } from "@polkadot/api";
import { foreignAssetBalance, registerAndFundAsset, type TestAsset } from "../../../../helpers";

const RELATIVE_PRICE_USDC_LIKE = 5_000_000_000_000_000_000_000_000_000_000n;
const BALANCE_RAW_SIX_DECIMAL_USDC = 1_380_000_000n;
const TEST_ASSET_ID = 777_888n;
const TEST_PARA = 424_242;

describeSuite({
  id: "D023704",
  title: "AssetRateConverter — treasury.spend uses runtime BalanceConverter (wide mul)",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let api: ApiPromise;
    let assetKind;

    const xcAsset: TestAsset = {
      id: TEST_ASSET_ID,
      location: {
        Xcm: { parents: 1, interior: { X1: { Parachain: TEST_PARA } } },
      },
      metadata: {
        name: "Test USD",
        symbol: "TUSD",
        decimals: 6n,
        isFrozen: false,
      },
      relativePrice: RELATIVE_PRICE_USDC_LIKE,
    };

    const treasuryForeignHold = BALANCE_RAW_SIX_DECIMAL_USDC * 10n;

    beforeAll(async function () {
      api = context.polkadotJs();
      assetKind = api.createType("FrameSupportTokensFungibleUnionOfNativeOrWithId", {
        WithId: xcAsset.id,
      });
    });

    it({
      id: "T01",
      title: "treasury.spend + payout succeeds when u128 product overflows (AssetRateConverter)",
      test: async function () {
        await registerAndFundAsset(context, xcAsset, treasuryForeignHold, TREASURY_ACCOUNT);

        expect(await foreignAssetBalance(context, TEST_ASSET_ID, TREASURY_ACCOUNT)).to.eq(
          treasuryForeignHold
        );

        const spendTx = api.tx.treasury.spend(
          assetKind,
          BALANCE_RAW_SIX_DECIMAL_USDC,
          ethan.address,
          null
        );
        await context.createBlock(await api.tx.sudo.sudo(spendTx).signAsync(alith), {
          allowFailures: false,
          expectEvents: [api.events.treasury.AssetSpendApproved],
        });

        expect((await api.query.treasury.spendCount()).toNumber()).to.eq(1);

        await context.createBlock(await api.tx.treasury.payout(0).signAsync(ethan), {
          allowFailures: false,
          expectEvents: [api.events.treasury.Paid],
        });

        expect(
          await foreignAssetBalance(context, TEST_ASSET_ID, ethan.address as `0x${string}`)
        ).to.eq(BALANCE_RAW_SIX_DECIMAL_USDC);

        expect(await foreignAssetBalance(context, TEST_ASSET_ID, TREASURY_ACCOUNT)).to.eq(
          treasuryForeignHold - BALANCE_RAW_SIX_DECIMAL_USDC
        );
      },
    });
  },
});

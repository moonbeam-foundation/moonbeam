import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, BALTATHAR_ADDRESS, alith, baltathar } from "@moonwall/util";
import type { u128 } from "@polkadot/types";
import type { ApiPromise } from "@polkadot/api";
import type { PalletAssetsAssetAccount, PalletAssetsAssetDetails } from "@polkadot/types/lookup";
import { expectSystemEvent, mockOldAssetBalance } from "../../../../helpers";

const ARBITRARY_ASSET_ID = 42n;

describeSuite({
  id: "D020001",
  title: "Pallet Asset Rate",
  foundationMethods: "dev",
  testCases: ({ context, log, it }) => {
    let assetId: u128;
    let api: ApiPromise;
    beforeAll(async () => {
        api = context.polkadotJs();
    });

    it({
      id: "T01",
      title: "Create, update and remove asset rate for Native asset",
      test: async function () {

        // Create native asset rate
        const assetKind = api.createType("FrameSupportTokensFungibleUnionOfNativeOrWithId", "Native");
        const createRate = api.tx.assetRate.create(
          assetKind,
          api.createType("u128", 1n)
        );
        const sudoCall1 = api.tx.sudo.sudo(createRate);
        const block = await context.createBlock(sudoCall1, { allowFailures: false });

        await expectSystemEvent(block.block.hash, "assetRate", "AssetRateCreated", context);

        // Update native asset rate
        const updateRate = api.tx.assetRate.update(
          assetKind,
          api.createType("u128", 2n)
        );
        const sudoCall2 = api.tx.sudo.sudo(updateRate);
        const block2 = await context.createBlock(sudoCall2, { allowFailures: false });

        await expectSystemEvent(block2.block.hash, "assetRate", "AssetRateUpdated", context);

        // Remove native asset rate
        const removeRate = api.tx.assetRate.remove(assetKind);
        const sudoCall3 = api.tx.sudo.sudo(removeRate);
        const block3 = await context.createBlock(sudoCall3, { allowFailures: false });

        await expectSystemEvent(block3.block.hash, "assetRate", "AssetRateRemoved", context);
      },
    });

    // Same test but with WithId asset
    it({
      id: "T02",
      title: "Create, update and remove asset rate for WithId asset",
      test: async function () {

        // Create WithId asset rate
        const assetKind = api.createType("FrameSupportTokensFungibleUnionOfNativeOrWithId", ARBITRARY_ASSET_ID);
        const createRate = api.tx.assetRate.create(
          assetKind,
          api.createType("u128", 1n)
        );
        const sudoCall1 = api.tx.sudo.sudo(createRate);
        const block = await context.createBlock(sudoCall1, { allowFailures: false });

        await expectSystemEvent(block.block.hash, "assetRate", "AssetRateCreated", context);

        // Update WithId asset rate
        const updateRate = api.tx.assetRate.update(
          assetKind,
          api.createType("u128", 2n)
        );
        const sudoCall2 = api.tx.sudo.sudo(updateRate);
        const block2 = await context.createBlock(sudoCall2, { allowFailures: false });

        await expectSystemEvent(block2.block.hash, "assetRate", "AssetRateUpdated", context);

        // Remove WithId asset rate
        const removeRate = api.tx.assetRate.remove(assetKind);
        const sudoCall3 = api.tx.sudo.sudo(removeRate);
        const block3 = await context.createBlock(sudoCall3, { allowFailures: false });

        await expectSystemEvent(block3.block.hash, "assetRate", "AssetRateRemoved", context);
      },
    });
  },
});

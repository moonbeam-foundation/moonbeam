import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite } from "@moonwall/cli";

import { sendCallAsPara, sovereignAccountOfSibling } from "../../../../helpers/xcm.js";
import { fundAccount } from "../../../../helpers/balances.js";
import { expectEvent } from "../../../../helpers/expect.js";

describeSuite({
  id: "D014112",
  title: "Gov intervention on created Foreign Assets",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const fundAmount = 100_000_000_000_000_000_000_000n;
    const assetId = 3;
    const assetLocation = {
      parents: 1,
      interior: {
        X3: [{ Parachain: 3000 }, { PalletInstance: 3 }, { GeneralIndex: 3 }],
      },
    };

    beforeAll(async () => {
      // Sibling Paras
      const siblingParas = [3000, 4000];
      const siblingParaSovereignAccounts = siblingParas.map((paraId) =>
        sovereignAccountOfSibling(context, paraId)
      );

      // Fund all accounts
      const fundAmount = 100_000_000_000_000_000_000_000n;
      for (const address of siblingParaSovereignAccounts) {
        await fundAccount(address as `0x${string}`, fundAmount, context);
      }

      // Create a foreign asset
      const createForeignAssetCall = context
        .polkadotJs()
        .tx.evmForeignAssets.createForeignAsset(assetId, assetLocation, 18, "TEST", "TEST");
      const block = await sendCallAsPara(createForeignAssetCall, 3000, context, fundAmount / 20n);
      await expectEvent(context, block.hash as `0x${string}`, "ForeignAssetCreated");
    });

    it({
      id: "T01",
      title: "Gov/Sudo should be able to freeze/unfreeze a foreign asset",
      test: async function () {
        const freezeForeignAssetCall = context
          .polkadotJs()
          .tx.evmForeignAssets.freezeForeignAsset(assetId, false);

        const sudoCall = context.polkadotJs().tx.sudo.sudo(freezeForeignAssetCall);
        const { block } = await context.createBlock(sudoCall);
        expectEvent(context, block.hash as `0x${string}`, "ForeignAssetFrozen");

        const unfreezeForeignAssetCall = context
          .polkadotJs()
          .tx.evmForeignAssets.unfreezeForeignAsset(assetId);
        const sudoCall2 = context.polkadotJs().tx.sudo.sudo(unfreezeForeignAssetCall);
        const { block: block2 } = await context.createBlock(sudoCall2);
        expectEvent(context, block2.hash as `0x${string}`, "ForeignAssetUnfrozen");
      },
    });

    it({
      id: "T02",
      title:
        "Gov/Sudo should be able to change XCM location and only new SiblingPara be able to manage",
      test: async function () {
        // Change location to Parachain 4000
        const newAssetLocation = {
          parents: 1,
          interior: {
            X3: [{ Parachain: 4000 }, { PalletInstance: 4 }, { GeneralIndex: 4 }],
          },
        };
        const changeForeignAssetLocationCall = context
          .polkadotJs()
          .tx.evmForeignAssets.changeXcmLocation(assetId, newAssetLocation);
        const sudoCall = context.polkadotJs().tx.sudo.sudo(changeForeignAssetLocationCall);
        const { block } = await context.createBlock(sudoCall);
        await expectEvent(context, block.hash as `0x${string}`, "ForeignAssetXcmLocationChanged");

        // // SiblingPara 3000 should not be able to manage the asset anymore
        const freezeForeignAssetCall = context
          .polkadotJs()
          .tx.evmForeignAssets.freezeForeignAsset(assetId, false);
        const block2 = await sendCallAsPara(
          freezeForeignAssetCall,
          3000,
          context,
          fundAmount / 20n
        );
        await expectEvent(context, block2.hash as `0x${string}`, "ForeignAssetFrozen");

        // SiblingPara 4000 should be able to manage the asset
        const block3 = await sendCallAsPara(
          freezeForeignAssetCall,
          4000,
          context,
          fundAmount / 20n
        );
        await expectEvent(context, block3.hash as `0x${string}`, "ForeignAssetFrozen");
      },
    });
  },
});

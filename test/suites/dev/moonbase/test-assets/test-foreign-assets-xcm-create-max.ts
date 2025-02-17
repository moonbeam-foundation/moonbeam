import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";

import { sendCallAsPara, sovereignAccountOfSibling } from "../../../../helpers/xcm.js";
import { fundAccount } from "../../../../helpers/balances.js";
import { expectEvent } from "../../../../helpers/expect.js";
import { SubmittableExtrinsic } from "@polkadot/api/types";
import { ISubmittableResult } from "@polkadot/types/types";

describeSuite({
  id: "D014114",
  title: "Creation of Foreign Assets via XCM",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const fundAmount = 100_000_000_000_000_000_000_000n;
    const maxForeignAssets = 256;

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
    });

    it({
      id: "T01",
      title: "Cannot create more than MAX foreign assets",
      test: async function () {
        // Range from 0 to 255
        const range = Array.from({ length: maxForeignAssets }, (_, i) => i);

        for (const i of range) {
          const string = i.toString().repeat(4);
          const assetLocation = {
            parents: 1,
            interior: {
              X3: [{ Parachain: 3000 }, { PalletInstance: i }, { GeneralIndex: i }],
            },
          };
          const assetCreationCall = context
            .polkadotJs()
            .tx.evmForeignAssets.createForeignAsset(i, assetLocation, 18, string, string);

          const sudoCall = context.polkadotJs().tx.sudo.sudo(assetCreationCall);

          await context.createBlock(sudoCall);
        }

        const totalAssets = await context.polkadotJs().query.evmForeignAssets.counterForAssetsById();
        expect(totalAssets.toNumber()).to.eq(256);

        const extraAssetLocation = {
          parents: 1,
          interior: {
            X3: [{ Parachain: 3000 }, { PalletInstance: 1 }, { GeneralIndex: 256 }],
          },
        };

        const extraAssetCreationCall = context
          .polkadotJs()
          .tx.evmForeignAssets.createForeignAsset(256, extraAssetLocation, 18, "BREAKS", "BREAKS");

        const { errorName } = await sendCallAsPara(
          extraAssetCreationCall,
          3000,
          context,
          fundAmount / 20n,
          true
        );

        expect(errorName).to.eq("TooManyForeignAssets");
      },
    });
  },
});

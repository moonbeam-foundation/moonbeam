import "@moonbeam-network/api-augment";
import { ALITH_ADDRESS, alith, beforeAll, describeSuite, expect } from "moonwall";

import { sovereignAccountOfSibling } from "../../../../helpers/xcm.js";
import { fundAccount } from "../../../../helpers/balances.js";
import { expectSubstrateEvent } from "../../../../helpers/expect.js";

// Maximum number of foreign assets that can be created (from runtime configuration)
const maxForeignAssets = 256;

describeSuite({
  id: "D020110",
  title: "Creation of Foreign Assets via XCM",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
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

        // Sign every sudo tx with an explicit, monotonically increasing nonce.
        // Letting `createBlock` query the nonce implicitly on each of the 256
        // tight iterations is racy: a query right after a block seals can read a
        // stale nonce and the tx is rejected with "1010: Transaction is outdated".
        let nonce = (
          await context.polkadotJs().rpc.system.accountNextIndex(ALITH_ADDRESS)
        ).toNumber();

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

          const sudoCall = await context
            .polkadotJs()
            .tx.sudo.sudo(assetCreationCall)
            .signAsync(alith, { nonce: nonce++ });

          const block = await context.createBlock(sudoCall);
          await expectSubstrateEvent(block, "evmForeignAssets", "ForeignAssetCreated");
        }

        const totalAssets = await context
          .polkadotJs()
          .query.evmForeignAssets.counterForAssetsById();
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

        // Creating a 257th foreign asset must not increase the total beyond MaxForeignAssets.
        const sudoExtra = await context
          .polkadotJs()
          .tx.sudo.sudo(extraAssetCreationCall)
          .signAsync(alith, { nonce: nonce++ });
        await context.createBlock(sudoExtra);

        const totalAfter = await context.polkadotJs().query.evmForeignAssets.counterForAssetsById();
        expect(totalAfter.toNumber()).to.eq(maxForeignAssets);
      },
    });
  },
});

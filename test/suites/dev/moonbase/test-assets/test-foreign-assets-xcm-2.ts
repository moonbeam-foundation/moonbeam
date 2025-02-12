import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";

import { sendCallAsPara, sovereignAccountOfSibling } from "../../../../helpers/xcm.js";
import { fundAccount, getReservedBalance } from "../../../../helpers/balances.js";
import { expectEvent } from "../../../../helpers/expect.js";

describeSuite({
  id: "D014111",
  title: "Creation Deposits for Foreign Assets via XCM",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const fundAmount = 100_000_000_000_000_000_000_000n;
    const assetId = 2;

    beforeAll(async () => {
      // Sibling Paras
      const siblingParas = [2000, 3333];
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
      title: "SiblingPara should reserve 100 tokens when creating a foreign asset",
      test: async function () {
        const assetLocation = {
          parents: 1,
          interior: {
            X3: [{ Parachain: 2000 }, { PalletInstance: 2 }, { GeneralIndex: 2 }],
          },
        };

        const reservedBalanceBefore = await getReservedBalance(
          sovereignAccountOfSibling(context, 2000) as `0x${string}`,
          context
        );
        expect(reservedBalanceBefore).to.eq(0n);

        const createForeignAssetCall = context
          .polkadotJs()
          .tx.evmForeignAssets.createForeignAsset(assetId, assetLocation, 18, "TEST", "TEST");

        const block = await sendCallAsPara(createForeignAssetCall, 2000, context, fundAmount / 20n);
        await expectEvent(context, block.hash as `0x${string}`, "ForeignAssetCreated");

        const reservedBalanceAfter = await getReservedBalance(
          sovereignAccountOfSibling(context, 2000) as `0x${string}`,
          context
        );

        expect(reservedBalanceAfter).to.eq(100_000_000_000_000_000_000n);
      },
    });
  },
});

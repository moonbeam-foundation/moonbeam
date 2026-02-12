import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";

import { sendCallAsPara, sovereignAccountOfSibling } from "../../../../helpers/xcm.js";
import { fundAccount, getFreeBalance, getReservedBalance } from "../../../../helpers/balances.js";
import { expectSystemEvent } from "../../../../helpers/expect.js";

describeSuite({
  id: "D020109",
  title: "Costs of creating a Foreign Asset via XCM",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    const paraId = 4444;
    let paraSovereignAccount: `0x${string}`;
    const depositAmount = 100_000_000_000_000_000_000n; // 100 tokens
    // Generous fee budget to accommodate upstream XCM weight changes.
    const feeBudget = 10_000_000_000_000_000_000_000n; // 10_000 GLMR
    const fundAmount = feeBudget + depositAmount;

    const assetId = 1;
    const assetLocation = {
      parents: 1,
      interior: {
        X3: [{ Parachain: paraId }, { PalletInstance: 1 }, { GeneralIndex: 1 }],
      },
    };

    beforeAll(async () => {
      paraSovereignAccount = sovereignAccountOfSibling(context, paraId) as `0x${string}`;
      await fundAccount(paraSovereignAccount, fundAmount, context);
    });

    it({
      id: "T01",
      title: "Account with right amount should be able to pay deposit & fees",
      test: async function () {
        const balanceBefore = await getFreeBalance(paraSovereignAccount, context);
        expect(balanceBefore).to.equal(fundAmount);
        const reservedBefore = await getReservedBalance(paraSovereignAccount, context);
        expect(reservedBefore).to.equal(0n);
        const createForeignAssetCall = context
          .polkadotJs()
          .tx.evmForeignAssets.createForeignAsset(assetId, assetLocation, 18, "TEST", "TEST");

        const { blockRes } = await sendCallAsPara(
          createForeignAssetCall,
          paraId,
          context,
          feeBudget
        );

        await expectSystemEvent(
          blockRes.block.hash,
          "evmForeignAssets",
          "ForeignAssetCreated",
          context
        );

        const balanceAfter = await getFreeBalance(paraSovereignAccount, context);
        const reservedAfter = await getReservedBalance(paraSovereignAccount, context);

        // Deposit should be reserved on the sovereign account.
        expect(reservedAfter).to.equal(depositAmount);
        // Some fee should have been paid, so free balance must decrease.
        expect(balanceAfter).to.be.lessThan(balanceBefore);
      },
    });
  },
});

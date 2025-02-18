import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";

import {
  sendCallAsDescendedOrigin,
  sendCallAsPara,
  sovereignAccountOfSibling,
} from "../../../../helpers/xcm.js";
import { fundAccount, getFreeBalance } from "../../../../helpers/balances.js";
import { generateKeyringPair } from "@moonwall/util";
import { expectSubstrateEvent, expectSystemEvent } from "../../../../helpers/expect.js";

describeSuite({
  id: "D014117",
  title: "Costs of creating a Foreign Asset via XCM",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const paraId = 9999;
    let paraSovereignAccount;

    const feeAmount = 1_000_000_000_000_000_000_000n; // 1000 tokens
    const depositAmount = 100_000_000_000_000_000_000n; // 100 tokens
    const fundAmount = feeAmount + depositAmount;

    const assetId = 1;
    const assetLocation = {
      parents: 1,
      interior: {
        X3: [{ Parachain: paraId }, { PalletInstance: 1 }, { GeneralIndex: 1 }],
      },
    };

    beforeAll(async () => {
      paraSovereignAccount = sovereignAccountOfSibling(context, paraId) as `0x${string}`;
      await fundAccount(paraSovereignAccount as `0x${string}`, fundAmount, context);
    });

    it({
      id: "T01",
      title: "Cannot create if location already exists",
      test: async function () {
        const balanceBefore = await getFreeBalance(paraSovereignAccount, context);
        expect(balanceBefore).toMatchInlineSnapshot(`1100000000000000000000n`);

        const createForeignAssetCall = context
          .polkadotJs()
          .tx.evmForeignAssets.createForeignAsset(assetId, assetLocation, 18, "TEST", "TEST");

        const { blockRes } = await sendCallAsPara(
          createForeignAssetCall,
          paraId,
          context,
          feeAmount
        );

        await expectSystemEvent(
          blockRes.block.hash,
          "evmForeignAssets",
          "ForeignAssetCreated",
          context
        );

        const balanceAfter = await getFreeBalance(paraSovereignAccount, context);
        expect(balanceAfter).toMatchInlineSnapshot();
        expect(balanceBefore - balanceAfter).toMatchInlineSnapshot();
      },
    });
  },
});

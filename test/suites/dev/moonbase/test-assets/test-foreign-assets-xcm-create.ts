import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";

import {
  sendCallAsDescendedOrigin,
  sendCallAsPara,
  sovereignAccountOfSibling,
} from "../../../../helpers/xcm.js";
import { fundAccount } from "../../../../helpers/balances.js";
import { generateKeyringPair } from "@moonwall/util";
import { expectSubstrateEvent, expectSystemEvent } from "../../../../helpers/expect.js";

describeSuite({
  id: "D014115",
  title: "Creation of Foreign Assets via XCM",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const fundAmount = 100_000_000_000_000_000_000_000n;

    beforeAll(async () => {
      // Sibling Paras
      const siblingParas = [6000, 7000];
      const siblingParaSovereignAccounts = siblingParas.map((paraId) =>
        sovereignAccountOfSibling(context, paraId)
      );

      // Fund all accounts
      const fundAmount = 100_000_000_000_000_000_000_000n;
      for (const address of siblingParaSovereignAccounts) {
        await fundAccount(address as `0x${string}`, fundAmount, context);
      }

      // Create a foreign asset
      const assetId = 111;
      const assetLocation = {
        parents: 1,
        interior: {
          X3: [{ Parachain: 6000 }, { PalletInstance: 111 }, { GeneralIndex: 111 }],
        },
      };

      const createForeignAssetCall = context
        .polkadotJs()
        .tx.evmForeignAssets.createForeignAsset(assetId, assetLocation, 18, "TEST", "TEST");

      const { blockRes } = await sendCallAsPara(
        createForeignAssetCall,
        6000,
        context,
        fundAmount / 20n
      );
      await expectSystemEvent(
        blockRes.block.hash,
        "evmForeignAssets",
        "ForeignAssetCreated",
        context
      );
    });

    it({
      id: "T01",
      title: "Cannot create if location already exists",
      test: async function () {
        const assetId = 11;

        const assetLocation = {
          parents: 1,
          interior: {
            X3: [{ Parachain: 6000 }, { PalletInstance: 99 }, { GeneralIndex: 99 }],
          },
        };

        const createForeignAssetCall = context
          .polkadotJs()
          .tx.evmForeignAssets.createForeignAsset(assetId, assetLocation, 18, "TEST", "TEST");

        const { blockRes: block1 } = await sendCallAsPara(
          createForeignAssetCall,
          6000,
          context,
          fundAmount / 20n
        );
        await expectSystemEvent(
          block1.block.hash,
          "evmForeignAssets",
          "ForeignAssetCreated",
          context
        );

        const createForeignAssetCall2 = context
          .polkadotJs()
          .tx.evmForeignAssets.createForeignAsset(assetId + 1, assetLocation, 18, "TEST2", "TEST2");

        const { errorName } = await sendCallAsPara(
          createForeignAssetCall2,
          6000,
          context,
          fundAmount / 20n,
          true
        );
        expect(errorName).to.eq("LocationAlreadyExists");
      },
    });

    it({
      id: "T02",
      title: "Cannot create if Id already exists",
      test: async function () {
        const assetId = 21;
        const assetLocation = {
          parents: 1,
          interior: {
            X3: [{ Parachain: 6000 }, { PalletInstance: 21 }, { GeneralIndex: 21 }],
          },
        };

        const anotherAssetLocation = {
          parents: 1,
          interior: {
            X3: [{ Parachain: 6000 }, { PalletInstance: 22 }, { GeneralIndex: 22 }],
          },
        };

        const createForeignAssetCall = context
          .polkadotJs()
          .tx.evmForeignAssets.createForeignAsset(assetId, assetLocation, 18, "TEST3", "TEST3");

        const { blockRes } = await sendCallAsPara(
          createForeignAssetCall,
          6000,
          context,
          fundAmount / 20n
        );
        await expectSystemEvent(
          blockRes.block.hash,
          "evmForeignAssets",
          "ForeignAssetCreated",
          context
        );

        const createForeignAssetCall2 = context
          .polkadotJs()
          .tx.evmForeignAssets.createForeignAsset(
            assetId,
            anotherAssetLocation,
            18,
            "TEST4",
            "TEST4"
          );

        const { errorName } = await sendCallAsPara(
          createForeignAssetCall2,
          6000,
          context,
          fundAmount / 20n,
          true
        );

        expect(errorName).to.eq("AssetAlreadyExists");
      },
    });

    it({
      id: "T03",
      title: "Cannot create if location is outside of the origin/sibling para",
      test: async function () {
        const assetId = 31;

        const locationOutside = {
          parents: 1,
          interior: {
            X3: [{ Parachain: 5000 }, { PalletInstance: 1 }, { GeneralIndex: 1 }],
          },
        };

        const createForeignAssetCall = context
          .polkadotJs()
          .tx.evmForeignAssets.createForeignAsset(assetId, locationOutside, 18, "TEST5", "TEST5");

        const { errorName } = await sendCallAsPara(
          createForeignAssetCall,
          7000,
          context,
          fundAmount / 20n,
          true
        );

        expect(errorName).to.eq("LocationOutsideOfOrigin");
      },
    });

    it({
      id: "T04",
      title: "Cannot create if signed by normal account",
      test: async function () {
        const assetId = 41;

        const assetLocation = {
          parents: 1,
          interior: {
            X3: [{ Parachain: 6000 }, { PalletInstance: 41 }, { GeneralIndex: 41 }],
          },
        };

        const randomAccount = generateKeyringPair();

        const createForeignAssetCall = context
          .polkadotJs()
          .tx.evmForeignAssets.createForeignAsset(assetId, assetLocation, 18, "TEST6", "TEST6");

        const { errorName } = await sendCallAsDescendedOrigin(
          randomAccount.address as `0x${string}`,
          createForeignAssetCall,
          6000,
          context,
          fundAmount / 20n,
          true
        );

        expect(errorName).to.eq("BadOrigin");
      },
    });
  },
});

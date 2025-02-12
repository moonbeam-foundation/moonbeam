import "@moonbeam-network/api-augment";
import { afterEach, beforeAll, describeSuite, type DevModeContext, expect } from "@moonwall/cli";

import {
  XcmFragment,
  type RawXcmMessage,
  injectHrmpMessageAndSeal,
  sovereignAccountOfSibling,
} from "../../../../helpers/xcm.js";
import { fundAccount } from "../../../../helpers/balances.js";

describeSuite({
  id: "D014113",
  title: "Freezing and Unfreezing Foreign Assets via XCM",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    const fundAmount = 100_000_000_000_000_000_000_000n;
    const assetId = 4;
    const assetLocation = {
      parents: 1,
      interior: {
        X3: [{ Parachain: 3000 }, { PalletInstance: 3 }, { GeneralIndex: 3 }],
      },
    };
    

    beforeAll(async () => {
      // Sibling Paras
      const siblingParas = [3000,4000];
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
      const block = await sendCallAsPara(createForeignAssetCall, 3000, fundAmount / 20n, context);
      await expectEvent(context, block.hash as `0x${string}`, "ForeignAssetCreated");
    });

    afterEach(async () => {
      // Reset asset state and expect it to be active
      const assetByLocation = (await context.polkadotJs().query.evmForeignAssets.assetsByLocation(assetLocation)).toJSON();
      console.log('Asset by location:', assetByLocation);
      console.log(assetByLocation![1]);
      if (assetByLocation![1] != 'Active') {
        const unfreezeForeignAssetCall = context
          .polkadotJs()
          .tx.evmForeignAssets.unfreezeForeignAsset(assetId);
          const sudoCall = context.polkadotJs().tx.sudo.sudo(unfreezeForeignAssetCall);
        await context.createBlock(sudoCall);
      }
      const assetAfter = (await context.polkadotJs().query.evmForeignAssets.assetsByLocation(assetLocation)).toJSON();
      expect(assetAfter![1]).to.eq('Active');
    });

    it({
      id: "T01",
      title: "Should not be able to freeze if already frozen via XCM",
      test: async function () {

        const freezeForeignAssetCall = context
          .polkadotJs()
          .tx.evmForeignAssets.freezeForeignAsset(assetId, false);

        const block1 = await sendCallAsPara(freezeForeignAssetCall, 3000, fundAmount / 20n, context);
        await expectEvent(context, block1.hash as `0x${string}`, "ForeignAssetFrozen");

        const block2 = await sendCallAsPara(freezeForeignAssetCall, 3000, fundAmount / 20n, context);
        await expectNoEvent(context, block2.hash as `0x${string}`, "ForeignAssetFrozen");
      },
    });

    it({
      id: "T02",
      title: "Should not be able to freeze if already frozen via Sudo/Gov",
      test: async function () {

        const freezeForeignAssetCall = context
          .polkadotJs()
          .tx.evmForeignAssets.freezeForeignAsset(assetId, false);

        const sudoCall1 = context.polkadotJs().tx.sudo.sudo(freezeForeignAssetCall);
        const { block: block1 } = await context.createBlock(sudoCall1);
        await expectEvent(context, block1.hash as `0x${string}`, "ForeignAssetFrozen");

        await context.createBlock();

        const sudoCall2 = context.polkadotJs().tx.sudo.sudo(freezeForeignAssetCall);
        const { block: block2 } = await context.createBlock(sudoCall2);
        await expectNoEvent(context, block2.hash as `0x${string}`, "ForeignAssetFrozen");
      },
    });

    it({
      id: "T03",
      title: "Should not be able to freeze/unfreeze if asset does not exist",
      test: async function () {
        const freezeForeignAssetCall = context
          .polkadotJs()
          .tx.evmForeignAssets.freezeForeignAsset(255, false);

        const block = await sendCallAsPara(freezeForeignAssetCall, 3000, fundAmount / 20n, context);
        await expectNoEvent(context, block.hash as `0x${string}`, "ForeignAssetFrozen");

        const unfreezeForeignAssetCall = context
          .polkadotJs()
          .tx.evmForeignAssets.unfreezeForeignAsset(255);

        const block2 = await sendCallAsPara(unfreezeForeignAssetCall, 3000, fundAmount / 20n, context);
        await expectNoEvent(context, block2.hash as `0x${string}`, "ForeignAssetUnfrozen");
      },
    });

    it({
      id: "T04",
      title: "Should not be able to freeze/unfreeze if not owner",
      test: async function () {
        const freezeForeignAssetCall = context
          .polkadotJs()
          .tx.evmForeignAssets.freezeForeignAsset(assetId, false);

        const block = await sendCallAsPara(freezeForeignAssetCall, 4000, fundAmount / 20n, context);
        await expectNoEvent(context, block.hash as `0x${string}`, "ForeignAssetFrozen");

        const unfreezeForeignAssetCall = context
          .polkadotJs()
          .tx.evmForeignAssets.unfreezeForeignAsset(assetId);

        const block2 = await sendCallAsPara(unfreezeForeignAssetCall, 4000, fundAmount / 20n, context);
        await expectNoEvent(context, block2.hash as `0x${string}`, "ForeignAssetUnfrozen");
      },
    });
  },
});

async function expectEvent(context: DevModeContext, blockHash: `0x${string}`, eventName: string) {
  const apiAt = await context.polkadotJs().at(blockHash);
  const events = await apiAt.query.system.events();
  const event = events.find(({ event: { method } }) => method.toString() === eventName)!.event;
  expect(event).to.exist;
  return event;
}

async function expectNoEvent(context: DevModeContext, blockHash: `0x${string}`, eventName: string) {
  const apiAt = await context.polkadotJs().at(blockHash);
  const events = await apiAt.query.system.events();
  const event = events.find(({ event: { method } }) => method.toString() === eventName);
  expect(event).to.not.exist;
  return event;
}

const getPalletIndex = async (name: string, context: DevModeContext) => {
  const metadata = await context.polkadotJs().rpc.state.getMetadata();
  return metadata.asLatest.pallets
    .find(({ name: palletName }) => palletName.toString() === name)!
    .index.toNumber();
};

const sendCallAsPara = async (
  call: any,
  paraId: number,
  fungible: bigint = 10_000_000_000_000_000_000n, // Default 10 GLMR
  context: DevModeContext
) => {
  const encodedCall = call.method.toHex();
  const balancesPalletIndex = await getPalletIndex("Balances", context);

  const xcmMessage = new XcmFragment({
    assets: [
      {
        multilocation: {
          parents: 0,
          interior: {
            X1: { PalletInstance: balancesPalletIndex },
          },
        },
        fungible: fungible
      },
    ],
    weight_limit: {
      refTime: 40_000_000_000n,
      proofSize: 120_000n,
    },
  })
    .withdraw_asset()
    .buy_execution()
    .push_any({
      Transact: {
        originKind: "Xcm",
        requireWeightAtMost: {
          refTime: 20_089_165_000n,
          proofSize: 80_000n,
        },
        call: {
          encoded: encodedCall,
        },
      },
    })
    .as_v4();

  // Send an XCM and create block to execute it
  const block = await injectHrmpMessageAndSeal(context, paraId, {
    type: "XcmVersionedXcm",
    payload: xcmMessage,
  } as RawXcmMessage);

  return block;
}

import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, type DevModeContext, expect } from "@moonwall/cli";

import { generateKeyringPair } from "@moonwall/util";
import {
  XcmFragment,
  type RawXcmMessage,
  injectHrmpMessageAndSeal,
  descendOriginFromAddress20,
  sovereignAccountOfSibling,
} from "../../../../helpers/xcm.js";
import { fundAccount, getReservedBalance } from "../../../../helpers/balances.js";

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
      title: "Gov/Sudo should be able to change XCM location and only new SiblingPara be able to manage",
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
        const block2 = await sendCallAsPara(freezeForeignAssetCall, 3000, fundAmount / 20n, context);
        await expectNoEvent(context, block2.hash as `0x${string}`, "ForeignAssetFrozen");

        // SiblingPara 4000 should be able to manage the asset
        const block3 = await sendCallAsPara(freezeForeignAssetCall, 4000, fundAmount / 20n, context);
        await expectEvent(context, block3.hash as `0x${string}`, "ForeignAssetFrozen");
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

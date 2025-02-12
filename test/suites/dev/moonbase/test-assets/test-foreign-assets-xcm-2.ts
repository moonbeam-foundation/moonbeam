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

        const block = await sendCallAsPara(createForeignAssetCall, 2000, fundAmount / 20n, context);
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

async function expectEvent(context: DevModeContext, blockHash: `0x${string}`, eventName: string) {
  const apiAt = await context.polkadotJs().at(blockHash);
  const events = await apiAt.query.system.events();
  const event = events.find(({ event: { method } }) => method.toString() === eventName)!.event;
  expect(event).to.exist;
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

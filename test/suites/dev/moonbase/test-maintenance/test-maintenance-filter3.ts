import "@moonbeam-network/api-augment";
import {
  beforeAll,
  beforeEach,
  describeSuite,
  expect,
  execOpenTechCommitteeProposal,
} from "@moonwall/cli";
import {
  expectSystemEvent,
  fundAccount,
  getPalletIndex,
  injectHrmpMessage,
  type RawXcmMessage,
  sovereignAccountOfSibling,
  XcmFragment,
} from "../../../../helpers";

describeSuite({
  id: "D012003",
  title: "Maintenance Mode - Filter2",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const assetId = 1;
    const foreignParaId = 2000;
    const assetLocation = {
      parents: 1,
      interior: {
        X3: [{ Parachain: foreignParaId }, { PalletInstance: 1 }, { GeneralIndex: 1 }],
      },
    };

    beforeAll(async () => {
      const paraAddress = sovereignAccountOfSibling(context, foreignParaId) as `0x${string}`;
      await fundAccount(paraAddress, 1_000_000_000_000_000_000_000n, context);
    });

    beforeEach(async () => {
      await execOpenTechCommitteeProposal(
        context,
        context.polkadotJs().tx.maintenanceMode.enterMaintenanceMode()
      );
    });

    it({
      id: "T01",
      title: "should queue XCM messages until resuming operations",
      test: async () => {
        const createForeignAssetCall = context
          .polkadotJs()
          .tx.evmForeignAssets.createForeignAsset(assetId, assetLocation, 18, "TEST", "TEST");

        const { blockRes } = await sendCallAsParaUnchecked(
          createForeignAssetCall,
          foreignParaId,
          context,
          1_000_000_000_000_000_000n
        );

        // Check asset has not been created yet
        const assetInfo = await context.polkadotJs().query.evmForeignAssets.assetsById(assetId);
        expect(assetInfo.isNone).to.eq(true);

        // Turn maintenance off
        await execOpenTechCommitteeProposal(
          context,
          context.polkadotJs().tx.maintenanceMode.resumeNormalOperation()
        );

        // Expect the asset to be created
        const currentBlock = await context.polkadotJs().rpc.chain.getBlock();
        await expectSystemEvent(
          currentBlock.block.hash.toString(),
          "evmForeignAssets",
          "ForeignAssetCreated",
          context
        );
      },
    });
  },
});

const sendCallAsParaUnchecked = async (call, paraId, context, fungible) => {
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
        fungible: fungible,
      },
    ],
    weight_limit: {
      refTime: 40_000_000_000n,
      proofSize: 150_000n,
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

  await injectHrmpMessage(context, paraId, {
    type: "XcmVersionedXcm",
    payload: xcmMessage,
  } as RawXcmMessage);

  const blockRes = await context.createBlock();

  return { blockRes };
};

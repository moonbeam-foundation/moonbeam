import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";

import { BN } from "@polkadot/util";
import { alith, baltathar } from "@moonwall/util";
import {
  XcmFragment,
  RawXcmMessage,
  injectHrmpMessageAndSeal,
  sovereignAccountOfSibling,
} from "../../../helpers/xcm.js";

const foreign_para_id = 2000;

describeSuite({
  id: "D3414",
  title: "Mock XCM - receive horizontal transfer",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let assetId: string;
    let transferredBalance: bigint;
    let sovereignAddress: string;

    beforeAll(async () => {
      // registerAsset
      const { result } = await context.createBlock(
        context
          .polkadotJs()
          .tx.sudo.sudo(
            context
              .polkadotJs()
              .tx.assetManager.registerLocalAsset(
                baltathar.address,
                baltathar.address,
                true,
                new BN(1)
              )
          )
      );

      const eventsRegister = result?.events;

      // Look for assetId in events
      const event = eventsRegister!.find(({ event }) =>
        context.polkadotJs().events.assetManager.LocalAssetRegistered.is(event)
      )!;
      assetId = event.event.data.assetId.toHex();

      transferredBalance = 100000000000000n;

      // mint asset
      await context.createBlock(
        context
          .polkadotJs()
          .tx.localAssets.mint(assetId, alith.address, transferredBalance)
          .signAsync(baltathar),
        { allowFailures: false }
      );

      sovereignAddress = sovereignAccountOfSibling(context, 2000);

      // We first fund parachain 2000 sovreign account
      await context.createBlock(
        context.polkadotJs().tx.balances.transfer(sovereignAddress, transferredBalance),
        { allowFailures: false }
      );

      // transfer to para Id sovereign to emulate having sent the tokens
      await context.createBlock(
        context.polkadotJs().tx.localAssets.transfer(assetId, sovereignAddress, transferredBalance),
        { allowFailures: false }
      );
    });

    it({
      id: "T01",
      title: "Should NOT receive 10 Local Assets and DEV for fee with old reanchor",
      test: async function () {
        const ownParaId = (await context.polkadotJs().query.parachainInfo.parachainId()).toNumber();
        const metadata = await context.polkadotJs().rpc.state.getMetadata();
        const balancesPalletIndex = metadata.asLatest.pallets
          .find(({ name }) => name.toString() == "Balances")!
          .index.toNumber();

        const localAssetsPalletIndex = metadata.asLatest.pallets
          .find(({ name }) => name.toString() == "LocalAssets")!
          .index.toNumber();

        // We are charging 100_000_000 weight for every XCM instruction
        // We are executing 4 instructions
        // 100_000_000 * 4 * 50000 = 20000000000000
        // We are charging 20 micro DEV for this operation
        // The rest should be going to the deposit account
        const xcmMessage = new XcmFragment({
          assets: [
            {
              multilocation: {
                parents: 1,
                interior: {
                  X2: [{ Parachain: ownParaId }, { PalletInstance: balancesPalletIndex }],
                },
              },
              fungible: transferredBalance,
            },
            {
              multilocation: {
                parents: 1,
                interior: {
                  X2: [{ Parachain: ownParaId }, { PalletInstance: localAssetsPalletIndex }],
                },
              },
              fungible: transferredBalance,
            },
          ],
          weight_limit: new BN(4000000000),
          beneficiary: baltathar.address,
        })
          .withdraw_asset()
          .clear_origin()
          .buy_execution()
          .deposit_asset(2n)
          .as_v2();

        // Send an XCM and create block to execute it
        await injectHrmpMessageAndSeal(context, foreign_para_id, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        // Old reanchor does not work anymore so no reception of tokens
        const baltatharLocalTokBalance = await context
          .polkadotJs()
          .query.localAssets.account(assetId, baltathar.address);

        expect(baltatharLocalTokBalance.isNone).to.eq(true);
      },
    });
  },
});

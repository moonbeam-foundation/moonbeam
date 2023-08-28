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
  id: "D3412",
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
          .signAsync(baltathar)
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
      title: "Should receive 10 Local Asset tokens and sufficent DEV to pay for fee",
      test: async function () {
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
                parents: 0,
                interior: {
                  X1: { PalletInstance: balancesPalletIndex },
                },
              },
              fungible: transferredBalance,
            },
            {
              multilocation: {
                parents: 0,
                interior: {
                  X2: [
                    { PalletInstance: localAssetsPalletIndex },
                    { GeneralIndex: BigInt(assetId) },
                  ],
                },
              },
              fungible: transferredBalance,
            },
          ],
          weight_limit: new BN(4000000000),
          beneficiary: alith.address,
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

        // Make sure the state has ALITH's LOCAL parachain tokens
        const alithLocalTokBalance = (
          await context.polkadotJs().query.localAssets.account(assetId, alith.address)
        )
          .unwrap()
          .balance.toBigInt();

        expect(alithLocalTokBalance.toString()).to.eq(transferredBalance.toString());
      },
    });
  },
});

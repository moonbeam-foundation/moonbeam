import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";

import { BN, u8aToHex } from "@polkadot/util";
import { ParaId } from "@polkadot/types/interfaces";
import { alith, baltathar } from "@moonwall/util";
import { XcmFragment, RawXcmMessage, injectHrmpMessageAndSeal } from "../../../helpers/xcm.js";

import { expectOk } from "../../../helpers/expect.js";

const foreign_para_id = 2000;

describeSuite({
  id: "D3514",
  title: "Mock XCM - receive horizontal transfer",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let assetId: string;
    let paraId: ParaId;
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
      assetId = eventsRegister!
        .find(({ event: { section } }) => section.toString() === "assetManager")
        .event.data[0].toHex()
        .replace(/,/g, "");

      transferredBalance = 100000000000000n;

      // mint asset
      await context.createBlock(
        context
          .polkadotJs()
          .tx.localAssets.mint(assetId, alith.address, transferredBalance)
          .signAsync(baltathar)
      );

      paraId = context.polkadotJs().createType("ParaId", 2000);
      sovereignAddress = u8aToHex(
        new Uint8Array([...new TextEncoder().encode("sibl"), ...paraId.toU8a()])
      ).padEnd(42, "0");

      // We first fund parachain 2000 sovreign account
      await expectOk(
        context.createBlock(
          context.polkadotJs().tx.balances.transfer(sovereignAddress, transferredBalance)
        )
      );

      // transfer to para Id sovereign to emulate having sent the tokens
      await expectOk(
        context.createBlock(
          context
            .polkadotJs()
            .tx.localAssets.transfer(assetId, sovereignAddress, transferredBalance)
        )
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
          weight_limit: {
            refTime: 4000000000n,
            proofSize: 80000n,
          } as any,
          beneficiary: alith.address,
        })
          .withdraw_asset()
          .clear_origin()
          .buy_execution()
          .deposit_asset_v3(2n)
          .as_v3();

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

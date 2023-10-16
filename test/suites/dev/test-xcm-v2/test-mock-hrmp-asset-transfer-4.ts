import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";

import { BN } from "@polkadot/util";
import { KeyringPair } from "@polkadot/keyring/types";
import { generateKeyringPair } from "@moonwall/util";
import {
  XcmFragment,
  RawXcmMessage,
  injectHrmpMessageAndSeal,
  weightMessage,
  sovereignAccountOfSibling,
} from "../../../helpers/xcm.js";

const foreign_para_id = 2000;

describeSuite({
  id: "D3411",
  title: "Mock XCM - receive horizontal transfer of DEV with new reanchor",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let random: KeyringPair;
    let transferredBalance: bigint;
    let sovereignAddress: string;

    beforeAll(async () => {
      random = generateKeyringPair();
      sovereignAddress = sovereignAccountOfSibling(context, 2000);

      transferredBalance = 100000000000000n;

      // We first fund parachain 2000 sovreign account
      await context.createBlock(
        context.polkadotJs().tx.balances.transfer(sovereignAddress, transferredBalance),
        { allowFailures: false }
      );

      const balance = (
        await context.polkadotJs().query.system.account(sovereignAddress)
      ).data.free.toBigInt();
      expect(balance).to.eq(transferredBalance);
    });

    it({
      id: "T01",
      title: "Should receive MOVR from para Id 2000 with new reanchor logic",
      test: async function () {
        // Get Pallet balances index
        const metadata = await context.polkadotJs().rpc.state.getMetadata();
        const balancesPalletIndex = metadata.asLatest.pallets
          .find(({ name }) => name.toString() == "Balances")!
          .index.toNumber();

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
          ],
          weight_limit: new BN(4000000000),
          beneficiary: random.address,
        })
          .withdraw_asset()
          .clear_origin()
          .buy_execution()
          .deposit_asset()
          .as_v2();

        const chargedWeight = await weightMessage(
          context,
          context.polkadotJs().createType("StagingXcmVersionedXcm", xcmMessage)
        );
        // We are charging chargedWeight
        // chargedWeight * 50000 = chargedFee
        const chargedFee = chargedWeight * 50000n;

        // Send an XCM and create block to execute it
        await injectHrmpMessageAndSeal(context, foreign_para_id, {
          type: "StagingXcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        // We should expect sovereign balance to be 0, since we have transferred the full amount
        const balance = (
          await context.polkadotJs().query.system.account(sovereignAddress)
        ).data.free.toBigInt();
        expect(balance.toString()).to.eq(0n.toString());

        // In the case of the random address: we have transferred 100000000000000,
        // but chargedFee have been deducted
        // for weight payment
        const randomBalance = (
          await context.polkadotJs().query.system.account(random.address)
        ).data.free.toBigInt();
        const expectedRandomBalance = transferredBalance - chargedFee;
        expect(randomBalance).to.eq(expectedRandomBalance);
      },
    });
  },
});

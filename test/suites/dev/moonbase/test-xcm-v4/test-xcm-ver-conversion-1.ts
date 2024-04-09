import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";

import { KeyringPair } from "@polkadot/keyring/types";
import { generateKeyringPair } from "@moonwall/util";
import {
  XcmFragment,
  RawXcmMessage,
  injectHrmpMessageAndSeal,
  weightMessage,
  sovereignAccountOfSibling,
} from "../../../../helpers/xcm.js";

const foreign_para_id = 2000;

describeSuite({
  id: "D014128",
  title: "XCM Moonbase: version compatibility",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let transferredBalance: bigint;
    let sovereignAddress: string;
    let random: KeyringPair;

    beforeAll(async () => {
      random = generateKeyringPair();
      sovereignAddress = sovereignAccountOfSibling(context, 2000);
      transferredBalance = 100000000000000n;

      await context.createBlock(
        context.polkadotJs().tx.balances.transferAllowDeath(sovereignAddress, transferredBalance),
        { allowFailures: false }
      );

      const balance = (
        await context.polkadotJs().query.system.account(sovereignAddress)
      ).data.free.toBigInt();
      expect(balance).to.eq(transferredBalance);
    });

    it({
      id: "T01",
      title: "Should execute v4 message",
      test: async function () {
        const metadata = await context.polkadotJs().rpc.state.getMetadata();
        const balancesPalletIndex = metadata.asLatest.pallets
          .find(({ name }) => name.toString() == "Balances")!
          .index.toNumber();

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
          weight_limit: {
            refTime: 8000000000,
            proofSize: 110000n,
          },
          beneficiary: random.address,
        })
          .withdraw_asset()
          .clear_origin()
          .buy_execution()
          .deposit_asset_v3()
          .as_v4();

        const chargedWeight = await weightMessage(
          context,
          context.polkadotJs().createType("XcmVersionedXcm", xcmMessage)
        );

        const chargedFee = chargedWeight * 50000n;

        await injectHrmpMessageAndSeal(context, foreign_para_id, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        const balance = (
          await context.polkadotJs().query.system.account(sovereignAddress)
        ).data.free.toBigInt();
        expect(balance.toString(), "Sovereign account not empty, transfer has failed").to.eq(
          0n.toString()
        );

        const randomBalance = (
          await context.polkadotJs().query.system.account(random.address)
        ).data.free.toBigInt();
        const expectedRandomBalance = transferredBalance - chargedFee;
        expect(randomBalance, "Balance not increased, transfer has failed").to.eq(
          expectedRandomBalance
        );
      },
    });
  },
});

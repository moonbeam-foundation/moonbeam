import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";

import { KeyringPair } from "@polkadot/keyring/types";
import { generateKeyringPair } from "@moonwall/util";
import {
  XcmFragment,
  RawXcmMessage,
  injectHrmpMessageAndSeal,
  sovereignAccountOfSibling,
} from "../../../../helpers/xcm.js";

const foreign_para_id = 2000;

describeSuite({
  id: "D014109",
  title: "Mock XCM - receive horizontal transfer of DEV",
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
        context.polkadotJs().tx.balances.transferAllowDeath(sovereignAddress, transferredBalance)
      );
      const balance = (
        await context.polkadotJs().query.system.account(sovereignAddress)
      ).data.free.toBigInt();
      expect(balance).to.eq(transferredBalance);
    });

    it({
      id: "T01",
      title: "Should NOT receive MOVR from para Id 2000 with old reanchor",
      test: async function () {
        const ownParaId = (await context.polkadotJs().query.parachainInfo.parachainId()).toNumber();
        // Get Pallet balances index
        const metadata = await context.polkadotJs().rpc.state.getMetadata();
        const balancesPalletIndex = metadata.asLatest.pallets
          .find(({ name }) => name.toString() == "Balances")!
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
          ],
          weight_limit: {
            refTime: 40000000000n,
            proofSize: 110000n,
          },
          beneficiary: random.address,
        })
          .withdraw_asset()
          .clear_origin()
          .buy_execution()
          .deposit_asset_v3()
          .as_v4();

        // Send an XCM and create block to execute it
        await injectHrmpMessageAndSeal(context, foreign_para_id, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        // The message should not have been succesfully executed, since old prefix is not supported
        // anymore
        const balance = (
          await context.polkadotJs().query.system.account(sovereignAddress)
        ).data.free.toBigInt();
        expect(balance.toString()).to.eq(transferredBalance.toString());

        // the random address does not receive anything
        const randomBalance = (
          await context.polkadotJs().query.system.account(random.address)
        ).data.free.toBigInt();
        expect(randomBalance).to.eq(0n);
      },
    });
  },
});

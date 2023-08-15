import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";

import { BN, u8aToHex } from "@polkadot/util";
import { KeyringPair } from "@polkadot/keyring/types";
import { ParaId } from "@polkadot/types/interfaces";
import { generateKeyringPair } from "@moonwall/util";
import {
  XcmFragment,
  RawXcmMessage,
  injectHrmpMessageAndSeal,
  weightMessage,
} from "../../../helpers/xcm.js";

import { expectOk } from "../../../helpers/expect.js";

const foreign_para_id = 2000;

describeSuite({
  id: "D3435",
  title: "XCM Moonriver: version compatibility",
  foundationMethods: "dev",
  chainType: "moonriver",
  testCases: ({ context, it, log }) => {
    let transferredBalance: bigint;
    let sovereignAddress: string;
    let random: KeyringPair;
    let paraId: ParaId;

    beforeAll(async () => {
      random = generateKeyringPair();
      paraId = context.polkadotJs().createType("ParaId", 2000) as any;
      sovereignAddress = u8aToHex(
        new Uint8Array([...new TextEncoder().encode("sibl"), ...paraId.toU8a()])
      ).padEnd(42, "0");

      transferredBalance = 100000000000000n;
      await expectOk(
        context.createBlock(
          context.polkadotJs().tx.balances.transfer(sovereignAddress, transferredBalance)
        )
      );
      const balance = (
        (await context.polkadotJs().query.system.account(sovereignAddress)) as any
      ).data.free.toBigInt();
      expect(balance).to.eq(transferredBalance);
    });

    it({
      id: "T01",
      title: "Should execute v2 message",
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
          weight_limit: new BN(8000000000),
          beneficiary: random.address,
        })
          .withdraw_asset()
          .clear_origin()
          .buy_execution()
          .deposit_asset()
          .as_v2();

        const chargedWeight = await weightMessage(
          context,
          context.polkadotJs().createType("XcmVersionedXcm", xcmMessage) as any
        );

        const chargedFee = chargedWeight * 50000n;

        await injectHrmpMessageAndSeal(context, foreign_para_id, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        const balance = (
          (await context.polkadotJs().query.system.account(sovereignAddress)) as any
        ).data.free.toBigInt();
        expect(balance.toString(), "Sovereign account not empty, transfer has failed").to.eq(
          0n.toString()
        );

        const randomBalance = (
          (await context.polkadotJs().query.system.account(random.address)) as any
        ).data.free.toBigInt();
        const expectedRandomBalance = transferredBalance - chargedFee;
        expect(randomBalance, "Balance not increased, transfer has failed").to.eq(
          expectedRandomBalance
        );
      },
    });
  },
});

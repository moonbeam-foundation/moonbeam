import "@moonbeam-network/api-augment";
import { KeyringPair } from "@polkadot/keyring/types";
import { ParaId } from "@polkadot/types/interfaces";
import { BN, u8aToHex } from "@polkadot/util";
import { expect } from "chai";
import { generateKeyringPair } from "../../util/accounts";
import {
  injectHrmpMessageAndSeal,
  RawXcmMessage,
  weightMessage,
  XcmFragment,
} from "../../util/xcm";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { expectOk } from "../../util/expect";

const foreign_para_id = 2000;

// TODO: Add more test case permutations
describeDevMoonbeam(
  "XCM Moonbase: version compatibility",
  (context) => {
    let random: KeyringPair;
    let paraId: ParaId;
    let transferredBalance: bigint;
    let sovereignAddress: string;

    before("Should send DEV to the parachain sovereign", async function () {
      random = generateKeyringPair();
      paraId = context.polkadotApi.createType("ParaId", 2000) as any;
      sovereignAddress = u8aToHex(
        new Uint8Array([...new TextEncoder().encode("sibl"), ...paraId.toU8a()])
      ).padEnd(42, "0");

      transferredBalance = 100000000000000n;
      await expectOk(
        context.createBlock(
          context.polkadotApi.tx.balances.transfer(sovereignAddress, transferredBalance)
        )
      );
      const balance = (
        (await context.polkadotApi.query.system.account(sovereignAddress)) as any
      ).data.free.toBigInt();
      expect(balance).to.eq(transferredBalance);
    });

    it("Should execute v2 message", async function () {
      const metadata = await context.polkadotApi.rpc.state.getMetadata();
      const balancesPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find(
        (pallet) => pallet.name === "Balances"
      ).index;

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
        context.polkadotApi.createType("XcmVersionedXcm", xcmMessage) as any
      );

      const chargedFee = chargedWeight * 50000n;

      await injectHrmpMessageAndSeal(context, foreign_para_id, {
        type: "XcmVersionedXcm",
        payload: xcmMessage,
      } as RawXcmMessage);

      const balance = (
        (await context.polkadotApi.query.system.account(sovereignAddress)) as any
      ).data.free.toBigInt();
      expect(balance.toString(), "Sovereign account not empty, transfer has failed").to.eq(
        0n.toString()
      );

      const randomBalance = (
        (await context.polkadotApi.query.system.account(random.address)) as any
      ).data.free.toBigInt();
      const expectedRandomBalance = transferredBalance - chargedFee;
      expect(randomBalance, "Balance not increased, transfer has failed").to.eq(
        expectedRandomBalance
      );
    });
  },
  "Legacy",
  "moonbase"
);

describeDevMoonbeam(
  "XCM Moonriver: version compatibility",
  (context) => {
    let random: KeyringPair;
    let paraId: ParaId;
    let transferredBalance: bigint;
    let sovereignAddress: string;

    before("Should send DEV to the parachain sovereign", async function () {
      random = generateKeyringPair();
      paraId = context.polkadotApi.createType("ParaId", 2000) as any;
      sovereignAddress = u8aToHex(
        new Uint8Array([...new TextEncoder().encode("sibl"), ...paraId.toU8a()])
      ).padEnd(42, "0");

      transferredBalance = 100000000000000n;
      await expectOk(
        context.createBlock(
          context.polkadotApi.tx.balances.transfer(sovereignAddress, transferredBalance)
        )
      );
      const balance = (
        (await context.polkadotApi.query.system.account(sovereignAddress)) as any
      ).data.free.toBigInt();
      expect(balance).to.eq(transferredBalance);
    });

    it("Should execute v2 message", async function () {
      const metadata = await context.polkadotApi.rpc.state.getMetadata();
      const balancesPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find(
        (pallet) => pallet.name === "Balances"
      ).index;

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
        context.polkadotApi.createType("XcmVersionedXcm", xcmMessage) as any
      );

      const chargedFee = chargedWeight * 50000n;

      await injectHrmpMessageAndSeal(context, foreign_para_id, {
        type: "XcmVersionedXcm",
        payload: xcmMessage,
      } as RawXcmMessage);

      const balance = (
        (await context.polkadotApi.query.system.account(sovereignAddress)) as any
      ).data.free.toBigInt();
      expect(balance.toString(), "Sovereign account not empty, transfer has failed").to.eq(
        0n.toString()
      );

      const randomBalance = (
        (await context.polkadotApi.query.system.account(random.address)) as any
      ).data.free.toBigInt();
      const expectedRandomBalance = transferredBalance - chargedFee;
      expect(randomBalance, "Balance not increased, transfer has failed").to.eq(
        expectedRandomBalance
      );
    });
  },
  "Legacy",
  "moonriver"
);

import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";

import { BN } from "@polkadot/util";
import { KeyringPair } from "@polkadot/keyring/types";
import { generateKeyringPair } from "@moonwall/util";
import {
  XcmFragment,
  RawXcmMessage,
  injectHrmpMessageAndSeal,
  descendOriginFromAddress20,
} from "../../../helpers/xcm.js";

describeSuite({
  id: "D3418",
  title: "Mock XCM - receive horizontal transact without withdraw",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let transferredBalance: bigint;
    let sendingAddress: `0x${string}`;
    let random: KeyringPair;

    beforeAll(async () => {
      const { originAddress, descendOriginAddress } = descendOriginFromAddress20(context);
      sendingAddress = originAddress;
      random = generateKeyringPair();
      transferredBalance = 10_000_000_000_000_000_000n;

      await context.createBlock(
        context.polkadotJs().tx.balances.transfer(descendOriginAddress, transferredBalance),
        { allowFailures: false }
      );

      const balance = (
        await context.polkadotJs().query.system.account(descendOriginAddress)
      ).data.free.toBigInt();
      expect(balance).to.eq(transferredBalance);
    });

    it({
      id: "T01",
      title: "Should fail to transact because barrier does not pass without withdraw",
      test: async function () {
        // Get Pallet balances index
        const metadata = await context.polkadotJs().rpc.state.getMetadata();
        const balancesPalletIndex = metadata.asLatest.pallets
          .find(({ name }) => name.toString() == "Balances")!
          .index.toNumber();

        const transferCall = context
          .polkadotJs()
          .tx.balances.transfer(random.address, transferredBalance / 10n);
        const transferCallEncoded = transferCall?.method.toHex();

        // We are going to test that we can receive a transact operation from parachain 1
        // using descendOrigin first but without withdraw
        const xcmMessage = new XcmFragment({
          assets: [
            {
              multilocation: {
                parents: 0,
                interior: {
                  X1: { PalletInstance: balancesPalletIndex },
                },
              },
              fungible: transferredBalance / 2n,
            },
          ],
          weight_limit: new BN(4000000000),
          descend_origin: sendingAddress,
        })
          .descend_origin()
          .buy_execution()
          .push_any({
            Transact: {
              originType: "SovereignAccount",
              requireWeightAtMost: new BN(1000000000),
              call: {
                encoded: transferCallEncoded,
              },
            },
          })
          .as_v2();

        // Send an XCM and create block to execute it
        await injectHrmpMessageAndSeal(context, 1, {
          type: "StagingXcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        // Make sure testAccount did not receive, because barrier prevented it
        const testAccountBalance = (
          await context.polkadotJs().query.system.account(random.address)
        ).data.free.toBigInt();

        expect(testAccountBalance).to.eq(0n);
      },
    });
  },
});

import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";

import type { KeyringPair } from "@polkadot/keyring/types";
import { generateKeyringPair } from "@moonwall/util";
import {
  XcmFragment,
  XCM_VERSIONS,
  type RawXcmMessage,
  injectHrmpMessageAndSeal,
  descendOriginFromAddress20,
} from "../../../../helpers/xcm.js";

describeSuite({
  id: "D024016",
  title: "Mock XCM - receive horizontal transact with two Descends",
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
        context
          .polkadotJs()
          .tx.balances.transferAllowDeath(descendOriginAddress, transferredBalance),
        { allowFailures: false }
      );

      const balance = (
        await context.polkadotJs().query.system.account(descendOriginAddress)
      ).data.free.toBigInt();
      expect(balance).to.eq(transferredBalance);
    });

    for (const xcmVersion of XCM_VERSIONS) {
      it({
        id: `T01-XCM-v${xcmVersion}`,
        title: `Should fail to transact because barrier only allows one descend origin (XCM v${xcmVersion})`,
        test: async function () {
          // Get Pallet balances index
          const metadata = await context.polkadotJs().rpc.state.getMetadata();
          const balancesPalletIndex = metadata.asLatest.pallets
            .find(({ name }) => name.toString() === "Balances")!
            .index.toNumber();

          const transferCall = context
            .polkadotJs()
            .tx.balances.transferAllowDeath(random.address, transferredBalance / 10n);
          const transferCallEncoded = transferCall?.method.toHex();

          // We are going to test that we can receive a transact operation from parachain 1
          // using 2 descendOrigin first
          let xcmMessage = new XcmFragment({
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
            weight_limit: {
              refTime: 4000000000n,
              proofSize: 110000n,
            },
            descend_origin: sendingAddress,
          })
            .descend_origin()
            .descend_origin()
            .withdraw_asset()
            .buy_execution()
            .push_any({
              Transact: {
                originKind: "SovereignAccount",
                requireWeightAtMost: {
                  refTime: 1000000000n,
                  proofSize: 80000n,
                },
                call: {
                  encoded: transferCallEncoded,
                },
              },
            });

          // Convert to appropriate XCM version
          if (xcmVersion === 3) {
            xcmMessage = xcmMessage.as_v3();
          } else if (xcmVersion === 4) {
            xcmMessage = xcmMessage.as_v4();
          } else if (xcmVersion === 5) {
            xcmMessage = xcmMessage.as_v5();
          }

          // Send an XCM and create block to execute it
          await injectHrmpMessageAndSeal(context, 1, {
            type: "XcmVersionedXcm",
            payload: xcmMessage,
          } as RawXcmMessage);

          // Make sure testAccount did not receive, because barrier prevented it
          const testAccountBalance = (
            await context.polkadotJs().query.system.account(random.address)
          ).data.free.toBigInt();

          expect(testAccountBalance).to.eq(0n);
        },
      });
    }
  },
});

import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";

import { KeyringPair } from "@polkadot/keyring/types";
import { generateKeyringPair } from "@moonwall/util";
import {
  XcmFragment,
  RawXcmMessage,
  injectHrmpMessageAndSeal,
  descendOriginFromAddress20,
} from "../../../../helpers/xcm.js";

describeSuite({
  id: "D014120",
  title: "Mock XCM - receive horizontal transact ETHEREUM (proxy)",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let transferredBalance: bigint;
    let sendingAddress: `0x${string}`;
    let descendAddress: `0x${string}`;
    let random: KeyringPair;

    beforeAll(async () => {
      const { originAddress, descendOriginAddress } = descendOriginFromAddress20(context);
      sendingAddress = originAddress;
      descendAddress = descendOriginAddress;
      random = generateKeyringPair();
      transferredBalance = 10_000_000_000_000_000_000n;

      // We first fund the descend origin derivated address
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

    it({
      id: "T01",
      title: "should fail to transact_through_proxy without proxy",
      test: async function () {
        // Get Pallet balances index
        const metadata = await context.polkadotJs().rpc.state.getMetadata();
        const balancesPalletIndex = metadata.asLatest.pallets
          .find(({ name }) => name.toString() == "Balances")!
          .index.toNumber();

        const amountToTransfer = transferredBalance / 10n;

        const xcmTransactions = [
          {
            V1: {
              gas_limit: 21000,
              fee_payment: {
                Auto: {
                  Low: null,
                },
              },
              action: {
                Call: random.address,
              },
              value: amountToTransfer,
              input: [],
              access_list: null,
            },
          },
          {
            V2: {
              gas_limit: 21000,
              action: {
                Call: random.address,
              },
              value: amountToTransfer,
              input: [],
              access_list: null,
            },
          },
        ];

        let feeAmount = 0n;

        // Gas limit + 2 db reads
        const targetXcmWeight = 1_325_000_000n + 100_000_000n;
        const targetXcmFee = targetXcmWeight * 50_000n;

        for (const xcmTransaction of xcmTransactions) {
          feeAmount += targetXcmFee;
          // TODO need to update lookup types for xcm ethereum transaction V2
          const transferCall = context
            .polkadotJs()
            .tx.ethereumXcm.transactThroughProxy(sendingAddress, xcmTransaction);
          const transferCallEncoded = transferCall?.method.toHex();

          // We are going to test that we can receive a transact operation from parachain 1
          // using descendOrigin first
          const xcmMessage = new XcmFragment({
            assets: [
              {
                multilocation: {
                  parents: 0,
                  interior: {
                    X1: { PalletInstance: balancesPalletIndex },
                  },
                },
                fungible: targetXcmFee,
              },
            ],
            weight_limit: {
              refTime: targetXcmWeight,
              proofSize: 110000n,
            },
            descend_origin: sendingAddress,
          })
            .descend_origin()
            .withdraw_asset()
            .buy_execution()
            .push_any({
              Transact: {
                originKind: "SovereignAccount",
                // 100_000 gas + 2 db read
                requireWeightAtMost: {
                  refTime: 575_000_000n,
                  proofSize: 80000n,
                },
                call: {
                  encoded: transferCallEncoded,
                },
              },
            })
            .as_v4();

          // Send an XCM and create block to execute it
          await injectHrmpMessageAndSeal(context, 1, {
            type: "XcmVersionedXcm",
            payload: xcmMessage,
          } as RawXcmMessage);

          // Make sure the state for the transfer recipient didn't change
          const testAccountBalance = (
            await context.polkadotJs().query.system.account(random.address)
          ).data.free.toBigInt();
          expect(testAccountBalance).to.eq(0n);

          // Make sure the descended address has been deducted fees once (in xcm-executor) but
          // transfered nothing.
          const descendOriginBalance = await context.viem().getBalance({ address: descendAddress });
          expect(BigInt(descendOriginBalance)).to.eq(transferredBalance - feeAmount);
        }
      },
    });
  },
});

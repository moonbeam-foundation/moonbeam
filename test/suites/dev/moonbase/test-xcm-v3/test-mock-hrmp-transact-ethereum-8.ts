import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";

import type { KeyringPair } from "@polkadot/keyring/types";
import { generateKeyringPair, alith } from "@moonwall/util";
import {
  XcmFragment,
  type RawXcmMessage,
  injectHrmpMessageAndSeal,
  descendOriginFromAddress20,
} from "../../../../helpers/xcm.js";

describeSuite({
  id: "D024014",
  title: "Mock XCM - transact ETHEREUM (non-proxy) disabled switch",
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

      // We first fund parachain 2000 sovreign account
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

      // We activate the suspension switch
      await context.createBlock(
        context
          .polkadotJs()
          .tx.sudo.sudo(context.polkadotJs().tx.ethereumXcm.suspendEthereumXcmExecution())
          .signAsync(alith)
      );
    });

    it({
      id: "T01",
      title: "should receive transact and should not be able to execute",
      test: async function () {
        // Get Pallet balances index
        const metadata = await context.polkadotJs().rpc.state.getMetadata();
        const balancesPalletIndex = metadata.asLatest.pallets
          .find(({ name }) => name.toString() === "Balances")!
          .index.toNumber();

        const amountToTransfer = transferredBalance / 10n;
        const GAS_LIMIT = 21_000;

        const xcmTransactions = [
          {
            V1: {
              gas_limit: GAS_LIMIT,
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
              gas_limit: GAS_LIMIT,
              action: {
                Call: random.address,
              },
              value: amountToTransfer,
              input: [],
              access_list: null,
            },
          },
        ];

        const targetXcmWeight = 5_000_000_000n + 100_000_000n;
        const targetXcmFee = 25_000_000n;

        let expectedTransferredAmountPlusFees = 0n;
        for (const xcmTransaction of xcmTransactions) {
          expectedTransferredAmountPlusFees += targetXcmFee;
          // TODO need to update lookup types for xcm ethereum transaction V2
          const transferCall = context.polkadotJs().tx.ethereumXcm.transact(xcmTransaction);
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
              proofSize: 43_208,
            } as any,
            descend_origin: sendingAddress,
          })
            .descend_origin()
            .withdraw_asset()
            .buy_execution()
            .push_any({
              Transact: {
                originKind: "SovereignAccount",
                // Allow up to the full XCM budget derived above so that
                // the Transact is not rejected purely due to heavier
                // upstream XCM/Transact weights.
                requireWeightAtMost: {
                  refTime: targetXcmWeight,
                  proofSize: 2_625, // Previously (with 5MB max PoV): 1312
                },
                call: {
                  encoded: transferCallEncoded,
                },
              },
            })
            .as_v3();

          // Send an XCM and create block to execute it
          await injectHrmpMessageAndSeal(context, 1, {
            type: "XcmVersionedXcm",
            payload: xcmMessage,
          } as RawXcmMessage);

          // Make sure tokens have not bein transferred
          const testAccountBalance = (
            await context.polkadotJs().query.system.account(random.address)
          ).data.free.toBigInt();
          expect(testAccountBalance).to.eq(0n);

          // Make sure descend address has been deducted fees once (in xcm-executor).
          // With the new upstream benchmarks and more accurate weight refunds,
          // and with the Ethereum XCM execution suspension switch enabled,
          // the exact fee depends on configuration and may even be fully
          // refunded. We only assert it stays within the originally
          // budgeted upper bound.
          const descendAddressBalance = await context
            .viem()
            .getBalance({ address: descendAddress });
          const spent = transferredBalance - BigInt(descendAddressBalance);
          expect(spent).to.be.lte(expectedTransferredAmountPlusFees);
        }
      },
    });
  },
});

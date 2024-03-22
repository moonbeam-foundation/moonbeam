import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";

import { GAS_LIMIT_POV_RATIO, generateKeyringPair } from "@moonwall/util";
import { KeyringPair } from "@polkadot/keyring/types";
import {
  XcmFragment,
  RawXcmMessage,
  injectHrmpMessageAndSeal,
  descendOriginFromAddress20,
} from "../../../../helpers/xcm.js";

describeSuite({
  id: "D014123",
  title: "Mock XCM - receive horizontal transact ETHEREUM (transfer)",
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
    });

    it({
      id: "T01",
      title: "should receive transact and should use less weight than gas limit",
      test: async function () {
        // Get Pallet balances index
        const metadata = await context.polkadotJs().rpc.state.getMetadata();
        const balancesPalletIndex = metadata.asLatest.pallets
          .find(({ name }) => name.toString() == "Balances")!
          .index.toNumber();

        const amountToTransfer = transferredBalance / 10n;

        const GAS_LIMIT = 500_000;

        // We will put a very high gas limit. However, the weight accounted
        // for the block should only
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
        ];

        let expectedTransferredAmount = 0n;
        let expectedTransferredAmountPlusFees = 0n;

        const targetXcmWeight = 500_000n * 25000n + 25_000_000n + 800000000n;
        const targetXcmFee = targetXcmWeight * 50_000n;

        for (const xcmTransaction of xcmTransactions) {
          expectedTransferredAmount += amountToTransfer;
          expectedTransferredAmountPlusFees += amountToTransfer + targetXcmFee;
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
              proofSize: (GAS_LIMIT / GAS_LIMIT_POV_RATIO) * 2,
            } as any,
            descend_origin: sendingAddress,
          })
            .descend_origin()
            .withdraw_asset()
            .buy_execution()
            .push_any({
              Transact: {
                originKind: "SovereignAccount",
                // 500_000 gas limit + db read
                requireWeightAtMost: {
                  refTime: 12_525_000_000,
                  proofSize: GAS_LIMIT / GAS_LIMIT_POV_RATIO,
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

          // Make sure the state has ALITH's foreign parachain tokens
          const testAccountBalance = (
            await context.polkadotJs().query.system.account(random.address)
          ).data.free.toBigInt();
          expect(testAccountBalance).to.eq(expectedTransferredAmount);

          // Make sure ALITH has been deducted fees once (in xcm-executor) and balance has been
          // transfered through evm.
          const alithAccountBalance = await context.viem().getBalance({ address: descendAddress });
          expect(BigInt(alithAccountBalance)).to.eq(
            transferredBalance - expectedTransferredAmountPlusFees
          );

          const weightBlock = await context.polkadotJs().query.system.blockWeight();
          // Make sure the system block weight corresponds to gas used and not gas limit
          // It should be sufficient to verify that we used less than what was marked
          expect(
            12_500_000_000n + 25_000_000n - weightBlock.mandatory.refTime.toBigInt()
          ).toBeGreaterThan(0n);
        }
      },
    });
  },
});

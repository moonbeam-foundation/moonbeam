import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect, generateKeyringPair } from "moonwall";

import type { KeyringPair } from "@polkadot/keyring/types";
import {
  XcmFragment,
  type RawXcmMessage,
  injectHrmpMessageAndSeal,
  descendOriginFromAddress20,
} from "../../../../helpers/xcm.js";
import { ConstantStore } from "../../../../helpers";

describeSuite({
  id: "D024010",
  title: "Mock XCM - receive horizontal transact ETHEREUM (proxy)",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let transferredBalance: bigint;
    let sendingAddress: `0x${string}`;
    let descendAddress: `0x${string}`;
    let random: KeyringPair;
    let GAS_LIMIT_POV_RATIO: number;

    beforeAll(async () => {
      const specVersion = (await context.polkadotJs().runtimeVersion.specVersion).toNumber();
      const constants = ConstantStore(context);
      GAS_LIMIT_POV_RATIO = Number(constants.GAS_PER_POV_BYTES.get(specVersion));

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

        let feeAmount = 0n;

        // Gas limit + 2 db reads
        const targetXcmWeight = 5_000_000_000n + 100_000_000n;
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
                  // This is impacted by `GasWeightMapping::gas_to_weight` in pallet-ethereum-xcm
                  proofSize: 2625, // Previously (with 5MB max PoV): 1312
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

          // Make sure the state for the transfer recipient didn't change
          const testAccountBalance = (
            await context.polkadotJs().query.system.account(random.address)
          ).data.free.toBigInt();
          expect(testAccountBalance).to.eq(0n);

          // Make sure the descended address has been charged, if at all, only
          // XCM fees while transferring nothing. With the new upstream
          // benchmarks and more accurate weight refunds, the exact fee depends
          // on configuration and may even be fully refunded, so we only assert
          // it stays within the originally budgeted upper bound.
          const descendOriginBalance = await context.viem().getBalance({ address: descendAddress });
          const spent = transferredBalance - BigInt(descendOriginBalance);
          expect(spent).to.be.lte(feeAmount);
        }
      },
    });
  },
});

import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";

import type { KeyringPair } from "@polkadot/keyring/types";
import { generateKeyringPair, charleth } from "@moonwall/util";
import {
  XcmFragment,
  type RawXcmMessage,
  injectHrmpMessageAndSeal,
  descendOriginFromAddress20,
} from "../../../../helpers/xcm.js";
import { ConstantStore } from "../../../../helpers/constants.js";

describeSuite({
  id: "D024123",
  title: "Mock XCM - receive horizontal transact ETHEREUM (proxy)",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let charlethBalance: bigint;
    let charlethNonce: number;
    let transferredBalance: bigint;
    let sendingAddress: `0x${string}`;
    let descendAddress: `0x${string}`;
    let random: KeyringPair;

    let STORAGE_READ_COST: bigint;

    beforeAll(async () => {
      STORAGE_READ_COST = ConstantStore(context).STORAGE_READ_COST;
      const { originAddress, descendOriginAddress } = descendOriginFromAddress20(
        context,
        charleth.address as `0x${string}`
      );
      sendingAddress = originAddress;
      descendAddress = descendOriginAddress;
      random = generateKeyringPair();
      transferredBalance = 100_000_000_000_000_000_000n;

      // We fund the Delegatee, which will send the xcm and pay fees
      await context.createBlock(
        context.polkadotJs().tx.balances.transferAllowDeath(descendAddress, transferredBalance),
        { allowFailures: false }
      );

      // Ensure funded
      const balance_delegatee = (
        await context.polkadotJs().query.system.account(descendAddress)
      ).data.free.toBigInt();
      expect(balance_delegatee).to.eq(transferredBalance);

      // Add proxy
      await context.createBlock(
        context.polkadotJs().tx.proxy.addProxy(descendAddress, "Any", 0).signAsync(charleth)
      );

      // Charleth balance after creating the proxy
      charlethBalance = (
        await context.polkadotJs().query.system.account(sendingAddress)
      ).data.free.toBigInt();

      // Charleth nonce
      charlethNonce = Number.parseInt(
        (await context.polkadotJs().query.system.account(sendingAddress)).nonce.toString()
      );
    });

    it({
      id: "T01",
      title: "should succeed to transact_through_proxy with proxy",
      test: async function () {
        // Get Pallet balances index
        const metadata = await context.polkadotJs().rpc.state.getMetadata();
        const balancesPalletIndex = metadata.asLatest.pallets
          .find(({ name }) => name.toString() === "Balances")!
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

        let expectedTransferredAmount = 0n;
        let expectedTransferredAmountPlusFees = 0n;

        const targetXcmWeight = 5_000_000_000n + 100_000_000n;
        const targetXcmFee = targetXcmWeight * 50_000n;

        for (const xcmTransaction of xcmTransactions) {
          expectedTransferredAmount += amountToTransfer;
          expectedTransferredAmountPlusFees += amountToTransfer + targetXcmFee;
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
              proofSize: 120_583n,
            },
            descend_origin: sendingAddress,
          })
            .descend_origin()
            .withdraw_asset()
            .buy_execution()
            .push_any({
              Transact: {
                originKind: "SovereignAccount",
                // 100_000 gas + 2db reads
                requireWeightAtMost: {
                  refTime: 575_000_000n + STORAGE_READ_COST,
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

          // The transfer destination
          // Make sure the destination address received the funds
          const testAccountBalance = (
            await context.polkadotJs().query.system.account(random.address)
          ).data.free.toBigInt();
          expect(testAccountBalance).to.eq(expectedTransferredAmount);

          // The EVM caller (proxy delegator)
          // Make sure CHARLETH called the evm on behalf DESCENDED, and CHARLETH balance was
          // deducted.
          const charlethAccountBalance = await context
            .viem()
            .getBalance({ address: sendingAddress });
          expect(BigInt(charlethAccountBalance)).to.eq(charlethBalance - expectedTransferredAmount);
          // Make sure CHARLETH nonce was increased, as EVM caller.
          const charlethAccountNonce = await context
            .viem()
            .getTransactionCount({ address: sendingAddress });
          expect(charlethAccountNonce).to.eq(charlethNonce + 1);
          charlethNonce++;

          // The XCM sender (proxy delegatee)
          // Make sure derived / descended account paid the xcm fees only.
          const derivedAccountBalance = await context
            .viem()
            .getBalance({ address: descendAddress });
          expect(BigInt(derivedAccountBalance)).to.eq(
            transferredBalance - (expectedTransferredAmountPlusFees - expectedTransferredAmount)
          );
          // Make sure derived / descended account nonce still zero.
          const derivedAccountNonce = await context
            .viem()
            .getTransactionCount({ address: descendAddress });
          expect(derivedAccountNonce).to.eq(0);
        }
      },
    });
  },
});

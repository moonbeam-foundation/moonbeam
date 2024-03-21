import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";

import { KeyringPair } from "@polkadot/keyring/types";
import { generateKeyringPair, charleth, alith } from "@moonwall/util";
import {
  XcmFragment,
  RawXcmMessage,
  injectHrmpMessageAndSeal,
  descendOriginFromAddress20,
} from "../../../../helpers/xcm.js";

describeSuite({
  id: "D014023",
  title: "Mock XCM - transact ETHEREUM (proxy) disabled switch",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let charlethBalance: bigint;
    let charlethNonce: number;
    let transferredBalance: bigint;
    let sendingAddress: `0x${string}`;
    let descendAddress: `0x${string}`;
    let random: KeyringPair;

    beforeAll(async () => {
      const { originAddress, descendOriginAddress } = descendOriginFromAddress20(
        context,
        charleth.address as `0x${string}`
      );
      sendingAddress = originAddress;
      descendAddress = descendOriginAddress;
      random = generateKeyringPair();
      transferredBalance = 10_000_000_000_000_000_000n;

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
      charlethNonce = parseInt(
        (await context.polkadotJs().query.system.account(sendingAddress)).nonce.toString()
      );

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
      title: "should fail to transact_through_proxy with proxy when disabled",
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

        let expectedTransferredAmount = 0n;
        let expectedTransferredAmountPlusFees = 0n;

        const targetXcmWeight = 1_325_000_000n + 100_000_000n;
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
                // 100_000 gas + 2db reads
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

          // The transfer destination
          // Make sure the destination address did not receive the funds
          const testAccountBalance = (
            await context.polkadotJs().query.system.account(random.address)
          ).data.free.toBigInt();
          expect(testAccountBalance).to.eq(0n);

          // The EVM caller (proxy delegator)
          // Make sure CHARLETH balance was not deducted.
          const charlethAccountBalance = await context
            .viem()
            .getBalance({ address: sendingAddress });
          expect(BigInt(charlethAccountBalance)).to.eq(charlethBalance);
          // Make sure CHARLETH nonce did not increase.
          const charlethAccountNonce = await context
            .viem()
            .getTransactionCount({ address: sendingAddress });
          expect(charlethAccountNonce).to.eq(charlethNonce);

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

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
  id: "D3420",
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
      title: "should receive transact and should be able to execute",
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

        const targetXcmWeight = 1_325_000_000n + 25_000_000n;
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
            weight_limit: new BN(targetXcmWeight.toString()),
            descend_origin: sendingAddress,
          })
            .descend_origin()
            .withdraw_asset()
            .buy_execution()
            .push_any({
              Transact: {
                originType: "SovereignAccount",
                // 21_000 gas limit + db read
                requireWeightAtMost: 550_000_000n,
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

          // Make sure the state has ALITH's foreign parachain tokens
          const testAccountBalance = (
            await context.polkadotJs().query.system.account(random.address)
          ).data.free.toBigInt();
          expect(testAccountBalance).to.eq(expectedTransferredAmount);

          // Make sure descend address has been deducted fees once (in xcm-executor) and balance
          // has been transfered through evm.
          const descendAccountBalance = await context
            .viem()
            .getBalance({ address: descendAddress });
          expect(BigInt(descendAccountBalance)).to.eq(
            transferredBalance - expectedTransferredAmountPlusFees
          );
        }
      },
    });
  },
});

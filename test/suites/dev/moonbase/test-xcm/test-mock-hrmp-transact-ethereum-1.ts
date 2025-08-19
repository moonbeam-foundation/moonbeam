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
  convertXcmFragmentToVersion,
  ConstantStore,
} from "../../../../helpers";

describeSuite({
  id: "D024019",
  title: "Mock XCM - receive horizontal transact ETHEREUM (transfer)",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let transferredBalance: bigint;
    let sendingAddress: `0x${string}`;
    let descendAddress: `0x${string}`;
    let random: KeyringPair;
    let STORAGE_READ_COST: bigint;
    let GAS_LIMIT_POV_RATIO: number;

    beforeAll(async () => {
      const specVersion = (await context.polkadotJs().runtimeVersion.specVersion).toNumber();
      const constants = ConstantStore(context);
      GAS_LIMIT_POV_RATIO = Number(constants.GAS_PER_POV_BYTES.get(specVersion));
      STORAGE_READ_COST = constants.STORAGE_READ_COST;
      const { originAddress, descendOriginAddress } = descendOriginFromAddress20(context);
      sendingAddress = originAddress;
      descendAddress = descendOriginAddress;
      random = generateKeyringPair();
      transferredBalance = 10_000_000_000_000_000_000n;

      // We first fund parachain 2000 sovereign account
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
        title: `should receive transact and should be able to execute (XCM v${xcmVersion})`,
        test: async function () {
          // Get initial balance
          const initialTestAccountBalance = (
            await context.polkadotJs().query.system.account(random.address)
          ).data.free.toBigInt();

          // Get initial descend account balance
          const initialDescendBalance = await context
            .viem()
            .getBalance({ address: descendAddress });

          // Get Pallet balances index
          const metadata = await context.polkadotJs().rpc.state.getMetadata();
          const balancesPalletIndex = metadata.asLatest.pallets
            .find(({ name }) => name.toString() === "Balances")!
            .index.toNumber();

          const amountToTransfer = transferredBalance / 10n;
          const TX_GAS_LIMIT = 21_000;

          const xcmTransactions = [
            {
              V1: {
                gas_limit: TX_GAS_LIMIT,
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
                gas_limit: TX_GAS_LIMIT,
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

          const targetXcmWeight = 5_000_000_000n + STORAGE_READ_COST;
          const targetXcmFee = targetXcmWeight * 50_000n;

          for (const xcmTransaction of xcmTransactions) {
            expectedTransferredAmount += amountToTransfer;
            expectedTransferredAmountPlusFees += amountToTransfer + targetXcmFee;
            // TODO need to update lookup types for xcm ethereum transaction V2
            const transferCall = context
              .polkadotJs()
              .tx.ethereumXcm.transact(xcmTransaction as any);
            const transferCallEncoded = transferCall?.method.toHex();

            // We are going to test that we can receive a transact operation from parachain 1
            // using descendOrigin first
            let xcmMessage = new XcmFragment({
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
              },
              descend_origin: sendingAddress,
            })
              .descend_origin()
              .withdraw_asset()
              .buy_execution()
              .push_any({
                Transact: {
                  originKind: "SovereignAccount",
                  // 21_000 gas limit + db read
                  requireWeightAtMost: {
                    refTime: 550_000_000n + STORAGE_READ_COST,
                    // This is impacted by `GasWeightMapping::gas_to_weight` in pallet-ethereum-xcm
                    proofSize: 2625, // Previously (with 5MB max PoV): 1312
                  },
                  call: {
                    encoded: transferCallEncoded,
                  },
                },
              });

            // Convert to appropriate XCM version
            xcmMessage = convertXcmFragmentToVersion(xcmMessage, xcmVersion);

            // Send an XCM and create block to execute it
            await injectHrmpMessageAndSeal(context, 1, {
              type: "XcmVersionedXcm",
              payload: xcmMessage,
            } as RawXcmMessage);

            // Make sure the state has ALITH's foreign parachain tokens
            const testAccountBalance = (
              await context.polkadotJs().query.system.account(random.address)
            ).data.free.toBigInt();
            expect(testAccountBalance - initialTestAccountBalance).to.eq(expectedTransferredAmount);

            // Make sure descend address has been deducted fees once (in xcm-executor) and balance
            // has been transfered through evm.
            const descendAccountBalance = await context
              .viem()
              .getBalance({ address: descendAddress });
            expect(BigInt(initialDescendBalance) - BigInt(descendAccountBalance)).to.eq(
              expectedTransferredAmountPlusFees
            );
          }
        },
      });
    }
  },
});

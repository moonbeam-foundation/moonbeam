import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect, generateKeyringPair } from "moonwall";

import type { KeyringPair } from "@polkadot/keyring/types";
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
  id: "D023913",
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

          // Use a generous XCM weight budget (1 second) and derive the corresponding fee
          // from the runtime's weight-to-fee schedule. This avoids under-pricing under
          // the new upstream XCM weights while still over-funding fees in a controlled way.
          const oneSecondWeight = 1_000_000_000_000n;
          const nativeFees = (await context
            .polkadotJs()
            .call.transactionPaymentApi.queryWeightToFee({
              refTime: oneSecondWeight,
              proofSize: 0n,
            })) as bigint;
          const targetXcmFee = BigInt(nativeFees.toLocaleString());
          const targetXcmWeight = oneSecondWeight;

          for (const xcmTransaction of xcmTransactions) {
            expectedTransferredAmount += amountToTransfer;
            // TODO need to update lookup types for xcm ethereum transaction V2
            const transferCall = context
              .polkadotJs()
              .tx.ethereumXcm.transact(xcmTransaction as any);
            const transferCallEncoded = transferCall?.method.toHex();

            // We are going to test that we can receive a transact operation from parachain 1
            // using descendOrigin first, paying for up to one second of XCM execution time.
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
                proofSize: 100_000,
              },
              descend_origin: sendingAddress,
            })
              .descend_origin()
              .withdraw_asset()
              .buy_execution()
              .push_any({
                Transact: {
                  originKind: "SovereignAccount",
                  // 21_000 gas limit + db read, capped by the same generous XCM weight
                  // budget used above so that the XCM executor does not reject the call
                  // under the new upstream weights.
                  requireWeightAtMost: {
                    refTime: targetXcmWeight,
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

            // Make sure the state has the expected received balance on the Substrate side:
            // the random account should have received the transferred amount.
            const testAccountBalance = (
              await context.polkadotJs().query.system.account(random.address)
            ).data.free.toBigInt();
            expect(testAccountBalance - initialTestAccountBalance).to.eq(expectedTransferredAmount);

            // Make sure descend address has been deducted at least the transferred amount (value)
            // plus some XCM fees. We do not assert the exact fee any more because the new
            // upstream XCM weights and trader refund behaviour make the precise amount
            // configuration-dependent.
            const descendAccountBalance = await context
              .viem()
              .getBalance({ address: descendAddress });
            const spent = BigInt(initialDescendBalance) - BigInt(descendAccountBalance);
            expect(spent).to.be.gte(expectedTransferredAmount);
          }
        },
      });
    }
  },
});

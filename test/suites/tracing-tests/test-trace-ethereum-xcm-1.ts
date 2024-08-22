import { beforeAll, customDevRpcRequest, describeSuite, expect } from "@moonwall/cli";
import {
  XcmFragment,
  injectHrmpMessageAndSeal,
  descendOriginFromAddress20,
  RawXcmMessage,
} from "../../helpers";
import { hexToNumber, Abi, encodeFunctionData } from "viem";

describeSuite({
  id: "T10",
  title: "Trace ethereum xcm #1",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let incremetorAddress: `0x${string}`;
    let incremetorABI: Abi;
    const transactionHashes: `0x${string}`[] = [];

    beforeAll(async () => {
      const { contractAddress, abi } = await context.deployContract!("Incrementor");
      incremetorAddress = contractAddress;
      incremetorABI = abi;

      const { originAddress, descendOriginAddress } = descendOriginFromAddress20(context);
      const sendingAddress = originAddress;
      const transferredBalance = 10_000_000_000_000_000_000n;

      // We first fund parachain 2000 sovreign account
      await context.createBlock(
        context
          .polkadotJs()
          .tx.balances.transferAllowDeath(descendOriginAddress, transferredBalance),
        { allowFailures: false }
      );

      // Get Pallet balances index
      const metadata = await context.polkadotJs().rpc.state.getMetadata();
      const balancesPalletIndex = metadata.asLatest.pallets
        .find(({ name }) => name.toString() == "Balances")!
        .index.toNumber();

      const xcmTransactions = [
        {
          V1: {
            gas_limit: 100000,
            fee_payment: {
              Auto: {
                Low: null,
              },
            },
            action: {
              Call: incremetorAddress,
            },
            value: 0n,
            input: encodeFunctionData({
              abi: incremetorABI,
              functionName: "incr",
              args: [],
            }),
            access_list: null,
          },
        },
        {
          V2: {
            gas_limit: 100000,
            action: {
              Call: incremetorAddress,
            },
            value: 0n,
            input: encodeFunctionData({
              abi: incremetorABI,
              functionName: "incr",
              args: [],
            }),
            access_list: null,
          },
        },
      ];

      for (const xcmTransaction of xcmTransactions) {
        const transferCall = context.polkadotJs().tx.ethereumXcm.transact(xcmTransaction);
        const transferCallEncoded = transferCall?.method.toHex();
        const xcmMessage = new XcmFragment({
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
          descend_origin: sendingAddress,
        })
          .descend_origin()
          .withdraw_asset()
          .buy_execution()
          .push_any({
            Transact: {
              originKind: "SovereignAccount",
              requireWeightAtMost: {
                refTime: 3000000000n,
                proofSize: 50000n,
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

        // Retrieve the stored ethereum transaction hash
        transactionHashes.push(
          (await context.viem().getBlock({ blockTag: "latest" })).transactions[0]
        );
      }
    });

    it({
      id: "T01",
      title: "should trace ethereum xcm transactions with debug_traceTransaction",
      test: async function () {
        for (const hash of transactionHashes) {
          const receipt = await context.viem().getTransactionReceipt({ hash });
          const trace = await customDevRpcRequest("debug_traceTransaction", [
            hash,
            { tracer: "callTracer" },
          ]);
          // We traced the transaction, and the traced gas used matches the one recorded
          // in the ethereum transaction receipt.
          expect(hexToNumber(trace.gasUsed)).to.eq(Number(receipt.gasUsed));
        }
      },
    });
  },
});

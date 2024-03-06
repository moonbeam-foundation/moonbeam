import { beforeAll, customDevRpcRequest, describeSuite, expect } from "@moonwall/cli";
import {
  XcmFragment,
  injectHrmpMessage,
  descendOriginFromAddress20,
  RawXcmMessage,
} from "../../helpers";
import { hexToNumber, Abi, encodeFunctionData } from "viem";

describeSuite({
  id: "T10",
  title: "Trace ethereum xcm #2: Multiple xcms in a block",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let incremetorAddress: `0x${string}`;
    let incremetorABI: Abi;
    const transactionHashes: `0x${string}`[] = [];

    beforeAll(async () => {
      const { contractAddress, abi } = await context.deployContract!("Incrementor");
      incremetorAddress = contractAddress;
      incremetorABI = abi;

      const { originAddress: originAddress1, descendOriginAddress: descendOriginAddress1 } =
        descendOriginFromAddress20(context);
      const sendingAddress1 = originAddress1;
      const { originAddress: originAddress2, descendOriginAddress: descendOriginAddress2 } =
        descendOriginFromAddress20(context, "0x0101010101010101010101010101010101010101", 2);
      const sendingAddress2 = originAddress2;
      const transferredBalance = 10_000_000_000_000_000_000n;

      // We first fund parachain 2000 sovreign account
      await context.createBlock(
        context
          .polkadotJs()
          .tx.balances.transferAllowDeath(descendOriginAddress1, transferredBalance),
        { allowFailures: false }
      );
      await context.createBlock(
        context
          .polkadotJs()
          .tx.balances.transferAllowDeath(descendOriginAddress2, transferredBalance),
        { allowFailures: false }
      );

      // Get Pallet balances index
      const metadata = await context.polkadotJs().rpc.state.getMetadata();
      const balancesPalletIndex = metadata.asLatest.pallets
        .find(({ name }) => name.toString() == "Balances")!
        .index.toNumber();

      const xcmTransaction = {
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
      };

      const transferCall = context.polkadotJs().tx.ethereumXcm.transact(xcmTransaction);
      const transferCallEncoded = transferCall?.method.toHex();
      for (const [paraId, sendingAddress] of [
        [1, sendingAddress1],
        [2, sendingAddress2],
      ]) {
        const xcmMessage = new XcmFragment({
          assets: [
            {
              multilocation: {
                parents: 0,
                interior: {
                  X1: { PalletInstance: balancesPalletIndex },
                },
              },
              fungible: transferredBalance,
            },
          ],
          weight_limit: {
            //refTime: 4000000000n,
            refTime: 4000000000n,
            //proofSize: 80000n,
            proofSize: 60000n,
          } as any,
          descend_origin: sendingAddress,
        })
          .descend_origin()
          .withdraw_asset()
          .buy_execution()
          .push_any({
            Transact: {
              originKind: "SovereignAccount",
              requireWeightAtMost: {
                //refTime: 3000000000n,
                refTime: 3000000000n,
                //proofSize: 50000n,
                proofSize: 30000n,
              },
              call: {
                encoded: transferCallEncoded,
              },
            },
          })
          .as_v3();

        // Send an XCM and create block to execute it
        await injectHrmpMessage(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);
      }
      await context.createBlock();

      const txHashes = (await context.viem().getBlock({ blockTag: "latest" })).transactions;
      transactionHashes.push(...txHashes);
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

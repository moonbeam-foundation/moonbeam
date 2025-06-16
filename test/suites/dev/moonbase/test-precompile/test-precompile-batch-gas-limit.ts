import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { PRECOMPILES, generateKeyringPair, sendRawTransaction } from "@moonwall/util";
import { encodeFunctionData, parseAbiItem } from "viem";
import { extractRevertReason } from "../../../../helpers";

// Casting of type in solidity is performing truncation:
// https://docs.soliditylang.org/en/latest/types.html#conversions-between-elementary-types
describeSuite({
  id: "D022810",
  title: "Precompile Batch - Overflowing gasLimit",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const randomAccount = generateKeyringPair();
    // This case can no longer be tested from frontier 0.9.23 because it is no longer possible to
    // enter a gas_limit that exceeds 65% of the block.
    it({
      id: "T01",
      title: "should get truncated and valid",
      test: async function () {
        // We are creating a fake function to override the argument type from uint64 to uint256
        const abiHeader = encodeFunctionData({
          abi: [parseAbiItem(["function batchAll(address[], uint256[], bytes[], uint64[])"])],
          functionName: "batchAll",
          args: [[], [], [], []],
        }).slice(0, 10);
        const secondAbiPart = encodeFunctionData({
          abi: [
            parseAbiItem(["function hackedbatchAll(address[], uint256[], bytes[], uint256[])"]),
          ],
          functionName: "hackedbatchAll",
          args: [
            [randomAccount.address as `0x${string}`],
            [3_000_000_000_000_000_000n],
            [],
            [2n ** 128n + 22_000n],
          ],
        }).slice(10);

        // each tx have a different gas limit to ensure it doesn't impact gas used
        const batchAllTx = await context.createTxn!({
          to: PRECOMPILES.Batch,
          gas: 1114112n,
          data: (abiHeader + secondAbiPart) as `0x${string}`,
        });

        const hash = await sendRawTransaction(context, batchAllTx);
        await context.createBlock();
        const batchAllReceipt = await context.viem().getTransactionReceipt({ hash });
        expect(batchAllReceipt.status).toBe("reverted");
        expect(await extractRevertReason(context, hash)).toContain("Value is too large for uint64");
      },
    });
  },
});

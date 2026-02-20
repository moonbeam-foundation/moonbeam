import "@moonbeam-network/api-augment";
import { createEthersTransaction, customDevRpcRequest, describeSuite, expect } from "moonwall";
import { EIP7623_GAS_CONSTANTS } from "../../../../helpers/fees";
import { EIP_7825_MAX_TRANSACTION_GAS_LIMIT } from "../../../../helpers";

describeSuite({
  id: "D021203",
  title: "Ethereum Transaction - Large Transaction",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    // EIP-7623: When sending pure data (all 0xFF bytes) with no execution,
    // the floor cost dominates: 21000 + nonzero_bytes * 40
    const { BASE_TX_COST, COST_FLOOR_PER_NON_ZERO_BYTE } = EIP7623_GAS_CONSTANTS;

    // Calculate exact max size that fits within EIP-7825 gas limit cap
    const exactMaxSize =
      (EIP_7825_MAX_TRANSACTION_GAS_LIMIT - BASE_TX_COST) / COST_FLOOR_PER_NON_ZERO_BYTE;

    // Buffer for potential overhead
    const maxSize = exactMaxSize - 2000n;

    it({
      id: "T01",
      title: "should accept txns up to known size",
      test: async function () {
        // Dynamically calculated: (16777216 - 21000) / 40 - 2000 = 416905
        expect(maxSize).to.equal(416905n); // max Ethereum TXN size with EIP-7623 floor cost and EIP-7825 cap
        // max_size - shanghai init cost - create cost
        const maxSizeShanghai = maxSize - 6474n;
        const data = ("0x" + "FF".repeat(Number(maxSizeShanghai))) as `0x${string}`;

        const rawSigned = await createEthersTransaction(context, {
          value: 0n,
          data,
          gasLimit: Number(EIP_7825_MAX_TRANSACTION_GAS_LIMIT),
        });

        const { result } = await context.createBlock(rawSigned);
        const receipt = await context
          .viem("public")
          .getTransactionReceipt({ hash: result!.hash as `0x${string}` });

        expect(receipt.status, "Junk txn should be accepted by RPC but reverted").toBe("reverted");
      },
    });

    it({
      id: "T02",
      title: "should reject txns which are too large to pay for",
      test: async function () {
        // Use exactMaxSize + 1 to ensure we exceed the gas limit
        const data = ("0x" + "FF".repeat(Number(exactMaxSize) + 1)) as `0x${string}`;

        const rawSigned = await createEthersTransaction(context, {
          value: 0n,
          data,
          gasLimit: Number(EIP_7825_MAX_TRANSACTION_GAS_LIMIT),
        });

        await expect(
          async () => await customDevRpcRequest("eth_sendRawTransaction", [rawSigned]),
          "RPC must reject before gossiping to prevent spam"
        ).rejects.toThrowError("intrinsic gas too low");
      },
    });
  },
});

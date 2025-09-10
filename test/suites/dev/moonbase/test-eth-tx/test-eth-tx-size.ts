import "@moonbeam-network/api-augment";
import { customDevRpcRequest, describeSuite, expect } from "@moonwall/cli";
import { EXTRINSIC_GAS_LIMIT, createEthersTransaction } from "@moonwall/util";

describeSuite({
  id: "D021303",
  title: "Ethereum Transaction - Large Transaction",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    // EIP-7623 gas calculation: gas = max(standard_cost, floor_cost)
    // Standard: 21000 + (zero_bytes * 4 + nonzero_bytes * 16) + execution_gas
    // Floor: 21000 + tokens * 10, where tokens = zero_bytes + nonzero_bytes * 4
    // For all 0xFF bytes (non-zero): tokens = nonzero_bytes * 4
    // Floor becomes: 21000 + nonzero_bytes * 40
    // Since we're sending pure data with no execution, floor cost dominates

    const BASE_TX_COST = 21000n;
    const TOKENS_PER_NONZERO_BYTE = 4n;
    const FLOOR_COST_PER_TOKEN = 10n;
    const FLOOR_COST_PER_NONZERO_BYTE = TOKENS_PER_NONZERO_BYTE * FLOOR_COST_PER_TOKEN; // 40

    // Calculate exact max size that fits within gas limit
    const exactMaxSize = (BigInt(EXTRINSIC_GAS_LIMIT) - BASE_TX_COST) / FLOOR_COST_PER_NONZERO_BYTE;

    // TODO: I'm not sure where this 2000 came from...
    const maxSize = exactMaxSize - 2000n;

    it({
      id: "T01",
      title: "should accept txns up to known size",
      test: async function () {
        // Dynamically calculated: (13000000 - 21000) / 40 - 2000 = 322475
        expect(maxSize).to.equal(322475n); // max Ethereum TXN size with EIP-7623 floor cost
        // max_size - shanghai init cost - create cost
        const maxSizeShanghai = maxSize - 6474n;
        const data = ("0x" + "FF".repeat(Number(maxSizeShanghai))) as `0x${string}`;

        const rawSigned = await createEthersTransaction(context, {
          value: 0n,
          data,
          gasLimit: EXTRINSIC_GAS_LIMIT,
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
          gasLimit: EXTRINSIC_GAS_LIMIT,
        });

        expect(
          async () => await customDevRpcRequest("eth_sendRawTransaction", [rawSigned]),
          "RPC must reject before gossiping to prevent spam"
        ).rejects.toThrowError("intrinsic gas too low");
      },
    });
  },
});

import "@moonbeam-network/api-augment";
import { customDevRpcRequest, describeSuite, expect } from "@moonwall/cli";
import { EXTRINSIC_GAS_LIMIT, createEthersTransaction } from "@moonwall/util";

describeSuite({
  id: "D021303",
  title: "Ethereum Transaction - Large Transaction",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    // With EIP-7623, non-zero bytes cost 40 gas (floor cost) instead of 16
    // Calculate max size based on floor cost
    const COST_FLOOR_PER_NON_ZERO_BYTE = 40n;
    const BASE_TX_COST = 21000n;
    // TODO: I'm not sure where this 2000 came from...
    const maxSize =
      (BigInt(EXTRINSIC_GAS_LIMIT) - BASE_TX_COST) / COST_FLOOR_PER_NON_ZERO_BYTE - 2000n;

    it({
      id: "T01",
      title: "should accept txns up to known size",
      test: async function () {
        // With EIP-7623 floor cost of 40 per non-zero byte: (13000000 - 21000) / 40 - 2000 = 322475
        expect(maxSize).to.equal(322475n); // our max Ethereum TXN size in bytes with EIP-7623
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
        const data = ("0x" + "FF".repeat(Number(maxSize) + 1)) as `0x${string}`;

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

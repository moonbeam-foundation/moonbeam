import "@moonbeam-network/api-augment";
import { createEthersTransaction, customDevRpcRequest, describeSuite, expect } from "moonwall";
import {
  DEFAULT_MAX_TX_INPUT_BYTES,
  EIP_7825_MAX_TRANSACTION_GAS_LIMIT,
} from "../../../../helpers";

describeSuite({
  id: "D021203",
  title: "Ethereum Transaction - Large Transaction",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    it({
      id: "T01",
      title: "should accept txns up to known size",
      test: async function () {
        // max_size - shanghai init cost - create cost
        const data = ("0x" + "FF".repeat(DEFAULT_MAX_TX_INPUT_BYTES - 6474)) as `0x${string}`;

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
      title: "should reject txns which exceed the size limit",
      test: async function () {
        const data = ("0x" + "FF".repeat(DEFAULT_MAX_TX_INPUT_BYTES)) as `0x${string}`;

        const rawSigned = await createEthersTransaction(context, {
          value: 0n,
          data,
          gasLimit: Number(EIP_7825_MAX_TRANSACTION_GAS_LIMIT),
        });

        const txSizeBytes = (rawSigned.length - 2) / 2;
        const errMsg = `oversized data: transaction size ${txSizeBytes} exceeds limit ${DEFAULT_MAX_TX_INPUT_BYTES}`;

        await expect(
          async () => await customDevRpcRequest("eth_sendRawTransaction", [rawSigned]),
          "RPC must reject oversized tx before gossiping"
        ).rejects.toThrow(errMsg);
      },
    });
  },
});

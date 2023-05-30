import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { customDevRpcRequest } from "../../../helpers/common.js";
import {
  ALITH_ADDRESS,
  EXTRINSIC_GAS_LIMIT,
  createEthersTxn,
  createRawTransaction,
} from "@moonwall/util";
import { parseGwei } from "viem";

describeSuite({
  id: "D1301",
  title: "Ethereum Transaction - Large Transaction",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    // TODO: I'm not sure where this 2000 came from...
    const maxSize = Math.floor((EXTRINSIC_GAS_LIMIT - 21000) / 16) - 2000;

    it({
      id: "T01",
      title: "should accept txns up to known size",
      test: async function () {
        expect(maxSize).to.equal(809187); // our max Ethereum TXN size in bytes
        const data = ("0x" + "FF".repeat(maxSize)) as `0x${string}`;

        const { rawSigned } = await createEthersTxn(context, {
          value: 0n,
          data,
          gasLimit: EXTRINSIC_GAS_LIMIT,
        });

        const { result } = await context.createBlock(rawSigned);
        const receipt = await context
          .viemClient("public")
          .getTransactionReceipt({ hash: result!.hash as `0x${string}` });

        expect(receipt.status, "Junk txn should be accepted by RPC but reverted").toBe("reverted");
      },
    });

    it({
      id: "T02",
      title: "should reject txns which are too large to pay for",
      test: async function () {
        const data = ("0x" + "FF".repeat(maxSize + 1)) as `0x${string}`;

        const { rawSigned } = await createEthersTxn(context, {
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

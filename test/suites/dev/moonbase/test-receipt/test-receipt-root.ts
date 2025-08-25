import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import { BALTATHAR_ADDRESS } from "@moonwall/util";
import { Receipt } from "eth-object";
import { BaseTrie as Trie } from "merkle-patricia-tree";
import * as RLP from "rlp";
import { type Log, encodeDeployData, toHex } from "viem";

describeSuite({
  id: "D023202",
  title: "Receipt root - With events",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      const deployData = encodeDeployData({
        abi: fetchCompiledContract("EventEmitter").abi,
        bytecode: fetchCompiledContract("EventEmitter").bytecode,
      });
      let nonce = 0;

      const legacyTransaction1 = await context.createTxn!({
        data: deployData,
        libraryType: "ethers",
        txnType: "legacy",
        nonce: nonce++,
      });

      const legacyTransaction2 = await context.createTxn!({
        to: BALTATHAR_ADDRESS,
        value: 1n,
        libraryType: "ethers",
        txnType: "legacy",
        nonce: nonce++,
      });

      const EIP2930Transaction1 = await context.createTxn!({
        data: deployData,
        libraryType: "ethers",
        txnType: "eip2930",
        nonce: nonce++,
      });

      const eip2930Transaction2 = await context.createTxn!({
        to: BALTATHAR_ADDRESS,
        value: 1n,
        libraryType: "ethers",
        nonce: nonce++,
        txnType: "eip2930",
      });

      const eip1559Transaction1 = await context.createTxn!({
        data: deployData,
        libraryType: "ethers",
        txnType: "eip1559",
        nonce: nonce++,
      });

      const eip1559Transaction2 = await context.createTxn!({
        to: BALTATHAR_ADDRESS,
        libraryType: "ethers",
        value: 1n,
        nonce: nonce++,
        txnType: "eip1559",
      });

      await context.createBlock([
        legacyTransaction1,
        legacyTransaction2,
        EIP2930Transaction1,
        eip2930Transaction2,
        eip1559Transaction1,
        eip1559Transaction2,
      ]);
    });

    //TODO: Fix when we have a better package than eth-object
    it({
      id: "T01",
      title: "Receipt root should match",
      // modifier: "skip",
      test: async function () {
        const block = await context.viem().getBlock({ blockNumber: 1n });
        const receipts = [];
        for (const txHash of block.transactions) {
          const receipt = await context
            .viem()
            .getTransactionReceipt({ hash: txHash as `0x${string}` });
          receipts.push(receipt);
        }
        // Verify we work with 6 receipts
        expect(receipts.length).toBe(6);
        // Verify we have a receipt of each type
        expect(receipts[0].type).toBe("legacy");
        expect(receipts[1].type).toBe("legacy");
        expect(receipts[2].type).toBe("eip2930");
        expect(receipts[3].type).toBe("eip2930");
        expect(receipts[4].type).toBe("eip1559");
        expect(receipts[5].type).toBe("eip1559");
        // Build the receipt trie.
        const tree = new Trie();
        await Promise.all(
          receipts.map(async (siblingReceipt, index) => {
            const innerReceipt: InnerReceipt = {
              logs: siblingReceipt.logs,
              // The MPT js library expects `status` to be a number, not the
              // web3 library `TransactionReceipt` boolean.
              status: siblingReceipt.status ? "0x1" : "0x0",
              cumulativeGasUsed: toHex(siblingReceipt.cumulativeGasUsed),
              logsBloom: siblingReceipt.logsBloom,
              type: convertTxnType(siblingReceipt.type as any),
            };

            // const serializedReceipt2 = serializeReceipt(innerReceipt); // This isn't working yet
            const siblingPath = RLP.encode(index);
            const serializedReceipt = Receipt.fromRpc(innerReceipt).serialize();
            return await tree.put(Buffer.from(siblingPath), serializedReceipt);
          })
        );
        // Onchain receipt root === Offchain receipt root
        expect(block.receiptsRoot).to.be.eq("0x" + tree.root.toString("hex"));
      },
    });
  },
});

interface InnerReceipt {
  logs: Log<bigint, number>[];
  status: string;
  cumulativeGasUsed: `0x${string}`;
  logsBloom: `0x${string}`;
  type: string;
}

// This is incomplete
function serializeReceipt(input: InnerReceipt) {
  const logs = input.logs.map((item) => {
    const topics = item.topics.map((topic) => Buffer.from(topic));
    return [Buffer.from(item.address), topics, Buffer.from(item.data)];
  });

  const receipt = [
    Buffer.from(input.status),
    Buffer.from(input.cumulativeGasUsed),
    Buffer.from(input.logsBloom),
    logs,
  ];
  return RLP.encode(receipt);
}

function convertTxnType(txnType: "legacy" | "eip2930" | "eip1559") {
  switch (txnType) {
    case "eip2930":
      return "0x1";
    case "eip1559":
      return "0x2";
    case "legacy":
      return "0x0";
    default:
      return "";
  }
}

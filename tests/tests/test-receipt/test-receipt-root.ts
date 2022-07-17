import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { alith, baltathar } from "../../util/accounts";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createContract, createTransfer } from "../../util/transactions";

import Tree from "merkle-patricia-tree";
import { Receipt } from "eth-object";
import { encode } from "eth-util-lite";
import { promisify } from "util";

describeDevMoonbeam(
  "Receipt root - With events",
  (context) => {
    before("Setup: Create block with multiple transaction types", async () => {
      // Legacy
      context.ethTransactionType = "Legacy";
      let legacyTransaction_1 = (
        await createContract(context, "EventEmitter", {
          from: alith.address,
          nonce: 0,
        })
      ).rawTx;
      let legacyTransaction_2 = await createTransfer(context, baltathar.address, 1, { nonce: 1 });
      // EIP2930
      context.ethTransactionType = "EIP2930";
      let EIP2930Transaction_1 = (
        await createContract(context, "EventEmitter", {
          from: alith.address,
          nonce: 2,
        })
      ).rawTx;
      let EIP2930Transaction_2 = await createTransfer(context, baltathar.address, 1, { nonce: 3 });
      // EIP1559
      context.ethTransactionType = "EIP1559";
      let EIP1559Transaction_1 = (
        await createContract(context, "EventEmitter", {
          from: alith.address,
          nonce: 4,
        })
      ).rawTx;
      let EIP1559Transaction_2 = await createTransfer(context, baltathar.address, 1, { nonce: 5 });
      await context.createBlock([
        legacyTransaction_1,
        legacyTransaction_2,
        EIP2930Transaction_1,
        EIP2930Transaction_2,
        EIP1559Transaction_1,
        EIP1559Transaction_2,
      ]);
    });

    it("Receipt root should match", async function () {
      const block = await context.web3.eth.getBlock(1);
      let receipts = [];
      for (const txHash of block.transactions) {
        const receipt = await context.web3.eth.getTransactionReceipt(txHash);
        receipts.push(receipt);
      }
      // Verify we work with 6 receipts
      expect(receipts.length).to.be.eq(6);
      // Verify we have a receipt of each type
      expect(receipts[0].type).to.be.eq("0x0");
      expect(receipts[1].type).to.be.eq("0x0");
      expect(receipts[2].type).to.be.eq("0x1");
      expect(receipts[3].type).to.be.eq("0x1");
      expect(receipts[4].type).to.be.eq("0x2");
      expect(receipts[5].type).to.be.eq("0x2");
      // Build the receipt trie.
      const tree = new Tree();
      await Promise.all(
        receipts.map((siblingReceipt, index) => {
          let innerReceipt = {
            logs: siblingReceipt.logs,
            // The MPT js library expects `status` to be a number, not the
            // web3 library `TransactionReceipt` boolean.
            status: siblingReceipt.status ? "0x1" : "0x0",
            cumulativeGasUsed: siblingReceipt.cumulativeGasUsed,
            logsBloom: siblingReceipt.logsBloom,
            type: siblingReceipt.type,
          };
          let siblingPath = encode(index);
          let serializedReceipt = Receipt.fromRpc(innerReceipt);
          serializedReceipt = serializedReceipt.serialize();
          let promisified = promisify(tree.put).bind(tree);
          return promisified(siblingPath, serializedReceipt);
        })
      );
      // Onchain receipt root == Offchain receipt root
      expect(block.receiptsRoot).to.be.eq("0x" + tree._root.toString("hex"));
    });
  },
  "Legacy"
);

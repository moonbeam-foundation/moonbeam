import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { BALTATHAR_ADDRESS, alith } from "@moonwall/util";

describeSuite({
  id: "D013203",
  title: "Receipt - Contract",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let txHash: string;
    let eventContract: `0x${string}`;

    beforeAll(async () => {
      const { contractAddress, hash } = await context.deployContract!("EventEmitter");
      eventContract = contractAddress;
      txHash = hash;
    });

    it({
      id: "T01",
      title: "Should generate receipt",
      test: async function () {
        // const block = await context.web3.eth.getBlock(1);
        const block = await context.viem().getBlock({ blockNumber: 1n });
        // const receipt = await context.web3.eth.getTransactionReceipt(txHash);
        const receipt = await context
          .viem()
          .getTransactionReceipt({ hash: txHash as `0x${string}` });

        expect(receipt.blockHash).toBe(block.hash);
        expect(receipt.blockNumber).toBe(block.number);
        expect(receipt.from).toBe(alith.address.toLowerCase()); // web3 rpc returns lowercase
        expect(receipt.logs.length).toBe(1);
        expect(receipt.logs[0].address).toBe(eventContract);
        expect(receipt.logs[0].blockHash).toBe(block.hash);
      },
    });

    it({
      id: "T02",
      title: "should calculate effective gas price",
      test: async function () {
        // With this configuration only half of the priority fee will be used,
        // as the max_fee_per_gas is 2GWEI and the base fee is 1GWEI.
        const maxFeePerGas = 10_000_000_000n * 2n;

        const rawTxn = await context.createTxn!({
          gas: 21000n,
          libraryType: "viem",
          maxFeePerGas: maxFeePerGas,
          maxPriorityFeePerGas: maxFeePerGas,
          to: BALTATHAR_ADDRESS,
          data: "0x",
          txnType: "eip1559",
        });
        await context.createBlock(rawTxn);

        const block = await context.viem().getBlock();
        const receipt = await context
          .viem()
          .getTransactionReceipt({ hash: block.transactions[0] as `0x${string}` });
        // The receipt should contain an effective gas price of 2GWEI.
        expect(receipt.effectiveGasPrice).to.be.eq(maxFeePerGas);
      },
    });
  },
});

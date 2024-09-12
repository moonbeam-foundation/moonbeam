import "@moonbeam-network/api-augment";
import { customDevRpcRequest, describeSuite, expect } from "@moonwall/cli";
import { BALTATHAR_ADDRESS, createViemTransaction } from "@moonwall/util";
import { DEFAULT_TXN_MAX_BASE_FEE } from "../../../../helpers";

describeSuite({
  id: "D012101",
  title: "Moon RPC Methods - moon_isBlockFinalized ",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    it({
      id: "T01",
      title: "should return as finalized when true",
      test: async function () {
        const blockHash = (await context.createBlock([], { finalize: true })).block.hash;
        const resp = await customDevRpcRequest("moon_isBlockFinalized", [blockHash]);
        expect(resp, "Block finalization status mismatch").toBe(true);
      },
    });

    it({
      id: "T02",
      title: "should return as unfinalized when false",
      test: async function () {
        const blockHash = (await context.createBlock([], { finalize: false })).block.hash;
        const resp = await customDevRpcRequest("moon_isBlockFinalized", [blockHash]);
        expect(resp, "Block finalization status mismatch").toBe(false);
      },
    });

    it({
      id: "T03",
      title: "should return as unfinalized when block not found",
      test: async function () {
        const blockHash = "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
        const resp = await customDevRpcRequest("moon_isBlockFinalized", [blockHash]);
        expect(resp, "Block finalization status mismatch").toBe(false);
      },
    });

    it({
      id: "T04",
      title: "should return as finalized when new block is true",
      test: async function () {
        const blockHash = (await context.createBlock([], { finalize: false })).block.hash;
        await context.createBlock([], { finalize: true });
        const resp = await customDevRpcRequest("moon_isBlockFinalized", [blockHash]);
        expect(resp, "Block finalization status mismatch").toBe(true);
      },
    });

    it({
      id: "T05",
      title: "should return as finalized when new block reorg happens",
      test: async function () {
        const blockHash = (await context.createBlock([], { finalize: false })).block.hash;
        await context.createBlock([], { finalize: false });
        await context.createBlock([], { finalize: true, parentHash: blockHash });

        const resp = await customDevRpcRequest("moon_isBlockFinalized", [blockHash]);
        expect(resp, "Block finalization status mismatch").toBe(true);
      },
    });

    it({
      id: "T06",
      title: "should return as finalized when true",
      test: async function () {
        await context.createBlock(
          await createViemTransaction(context, {
            to: BALTATHAR_ADDRESS,
            gas: 12_000_000n,
            gasPrice: BigInt(DEFAULT_TXN_MAX_BASE_FEE),
            value: 1_000_000n,
          }),
          { finalize: true }
        );

        const block = await context.viem().getBlock();
        const resp = await customDevRpcRequest("moon_isTxFinalized", [block.transactions[0]]);
        expect(resp, "Transaction finalization status mismatch").toBe(true);
      },
    });

    it({
      id: "T07",
      title: "should return as unfinalized when false",
      test: async function () {
        await context.createBlock(
          await createViemTransaction(context, {
            to: BALTATHAR_ADDRESS,
            gas: 12_000_000n,
            gasPrice: BigInt(DEFAULT_TXN_MAX_BASE_FEE),
            value: 1_000_000n,
          }),
          { finalize: false }
        );

        const block = await context.viem().getBlock();
        const resp = await customDevRpcRequest("moon_isTxFinalized", [block.transactions[0]]);
        expect(resp, "Transaction finalization status mismatch").toBe(false);
      },
    });

    it({
      id: "T08",
      title: "should return as unfinalized when txn not found",
      test: async function () {
        const txnHash = "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
        const resp = await customDevRpcRequest("moon_isTxFinalized", [txnHash]);
        expect(resp, "Transaction finalization status mismatch").toBe(false);
      },
    });

    it({
      id: "T09",
      title: "should return as finalized when new block is true",
      test: async function () {
        await context.createBlock(
          await createViemTransaction(context, {
            to: BALTATHAR_ADDRESS,
            gas: 12_000_000n,
            gasPrice: BigInt(DEFAULT_TXN_MAX_BASE_FEE),
            value: 1_000_000n,
          }),
          { finalize: false }
        );
        const block = await context.viem().getBlock();
        await context.createBlock([], { finalize: true });
        const resp = await customDevRpcRequest("moon_isTxFinalized", [block.transactions[0]]);
        expect(resp, "Transaction finalization status mismatch").toBe(true);
      },
    });

    it({
      id: "T10",
      title: "should return as finalized when new block reorg happens",
      test: async function () {
        const blockHash = (
          await context.createBlock(
            await createViemTransaction(context, {
              to: BALTATHAR_ADDRESS,
              gas: 12_000_000n,
              gasPrice: BigInt(DEFAULT_TXN_MAX_BASE_FEE),
              value: 1_000_000n,
            }),
            { finalize: false }
          )
        ).block.hash;

        const block = await context.viem().getBlock();
        await context.createBlock([], { finalize: false });
        await context.createBlock([], { finalize: true, parentHash: blockHash });
        const resp = await customDevRpcRequest("moon_isTxFinalized", [block.transactions[0]]);
        expect(resp, "Transaction finalization status mismatch").toBe(true);
      },
    });

    it({
      id: "T11",
      title: "should return latest synced block",
      test: async function () {
        const expected = await context.createBlock([], { finalize: true });
        const firstBlockHash = (await context.polkadotJs().rpc.chain.getBlockHash(0)).toHex();
        const resp = await customDevRpcRequest("moon_getEthSyncBlockRange", []);
        expect(resp[0], "First block hash").toBe(firstBlockHash);
        expect(resp[1], "Latest block hash").toBe(expected.block.hash);
      },
    });
  },
});

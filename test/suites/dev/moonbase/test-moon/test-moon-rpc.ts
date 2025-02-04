import "@moonbeam-network/api-augment/moonbase";
import { beforeEach, describeSuite, expect } from "@moonwall/cli";
import { BALTATHAR_ADDRESS, createViemTransaction } from "@moonwall/util";
import { types as BundledTypes } from "@moonbeam-network/types-bundle";
import { DEFAULT_TXN_MAX_BASE_FEE } from "../../../../helpers";
import { ApiPromise, WsProvider } from "@polkadot/api";

const createApi = async (endpoint: string) =>
  ApiPromise.create({
    provider: new WsProvider(endpoint),
    noInitWarn: true,
    throwOnConnect: false,
    throwOnUnknown: false,
    typesBundle: BundledTypes,
  });

describeSuite({
  id: "D012101",
  title: "Moon RPC Methods - moon_isBlockFinalized ",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let api: ApiPromise;

    beforeEach(async function () {
      const endpoint = `ws://127.0.0.1:${process.env.MOONWALL_RPC_PORT}`;
      api = await createApi(endpoint);
    });

    it({
      id: "T01",
      title: "should return as finalized when true",
      test: async function () {
        const blockHash = (await context.createBlock([], { finalize: true })).block.hash;
        const resp = await api.rpc.moon.isBlockFinalized(blockHash);
        expect(resp.isTrue, "Block finalization status mismatch").toBe(true);
      },
    });

    it({
      id: "T02",
      title: "should return as unfinalized when false",
      test: async function () {
        const blockHash = (await context.createBlock([], { finalize: false })).block.hash;
        const resp = await api.rpc.moon.isBlockFinalized(blockHash);
        expect(resp.isTrue, "Block finalization status mismatch").toBe(false);
      },
    });

    it({
      id: "T03",
      title: "should return as unfinalized when block not found",
      test: async function () {
        const blockHash = "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
        const resp = await api.rpc.moon.isBlockFinalized(blockHash);
        expect(resp.isTrue, "Block finalization status mismatch").toBe(false);
      },
    });

    it({
      id: "T04",
      title: "should return as finalized when new block is true",
      test: async function () {
        const blockHash = (await context.createBlock([], { finalize: false })).block.hash;
        await context.createBlock([], { finalize: true });
        const resp = await api.rpc.moon.isBlockFinalized(blockHash);
        expect(resp.isTrue, "Block finalization status mismatch").toBe(true);
      },
    });

    it({
      id: "T05",
      title: "should return as finalized when new block reorg happens",
      test: async function () {
        const blockHash = (await context.createBlock([], { finalize: false })).block.hash;
        await context.createBlock([], { finalize: false });
        await context.createBlock([], { finalize: true, parentHash: blockHash });

        const resp = await api.rpc.moon.isBlockFinalized(blockHash);
        expect(resp.isTrue, "Block finalization status mismatch").toBe(true);
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
        const resp = await api.rpc.moon.isTxFinalized(block.transactions[0]);
        expect(resp.isTrue, "Transaction finalization status mismatch").toBe(true);
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
        const resp = await api.rpc.moon.isTxFinalized(block.transactions[0]);
        expect(resp.isTrue, "Transaction finalization status mismatch").toBe(false);
      },
    });

    it({
      id: "T08",
      title: "should return as unfinalized when txn not found",
      test: async function () {
        const txnHash = "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
        const resp = await api.rpc.moon.isTxFinalized(txnHash);
        expect(resp.isTrue, "Transaction finalization status mismatch").toBe(false);
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
        const resp = await api.rpc.moon.isTxFinalized(block.transactions[0]);
        expect(resp.isTrue, "Transaction finalization status mismatch").toBe(true);
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
        const resp = await api.rpc.moon.isTxFinalized(block.transactions[0]);
        expect(resp.isTrue, "Transaction finalization status mismatch").toBe(true);
      },
    });

    it({
      id: "T11",
      title: "should return latest synced block",
      test: async function () {
        const expected = await context.createBlock([], { finalize: true });
        const firstBlockHash = (await context.polkadotJs().rpc.chain.getBlockHash(0)).toHex();
        const resp = await api.rpc.moon.getEthSyncBlockRange();
        expect(resp[0].toHex(), "First block hash").toBe(firstBlockHash);
        expect(resp[1].toHex(), "Latest block hash").toBe(expected.block.hash);
      },
    });
  },
});

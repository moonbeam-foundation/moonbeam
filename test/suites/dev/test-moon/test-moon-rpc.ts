import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { BALTATHAR_ADDRESS, createRawTransaction } from "@moonwall/util";
import { DEFAULT_TXN_MAX_BASE_FEE } from "../../../helpers/transactions.js";

describeSuite({
  id: "D2001",
  title: "Moon RPC Methods - moon_isBlockFinalized ",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    it({
      id: "T01",
      title: "should return as finalized when true",
      test: async function () {
        const blockHash = (await context.createBlock([], { finalize: true })).block.hash;
        const resp = await context.polkadotJs().rpc.moon.isBlockFinalized(blockHash);
        expect(resp.isTrue, "Block finalization status mismatch").to.be.true;
      },
    });

    it({
      id: "T02",
      title: "should return as unfinalized when false",
      test: async function () {
        const blockHash = (await context.createBlock([], { finalize: false })).block.hash;
        const resp = await context.polkadotJs().rpc.moon.isBlockFinalized(blockHash);
        expect(resp.isFalse, "Block finalization status mismatch").to.be.true;
      },
    });

    it({
      id: "T03",
      title: "should return as unfinalized when block not found",
      test: async function () {
        const blockHash = "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
        const resp = await context.polkadotJs().rpc.moon.isBlockFinalized(blockHash);
        expect(resp.isFalse, "Block finalization status mismatch").to.be.true;
      },
    });

    it({
      id: "T04",
      title: "should return as finalized when new block is true",
      test: async function () {
        const blockHash = (await context.createBlock([], { finalize: false })).block.hash;
        await context.createBlock([], { finalize: true });
        const resp = await context.polkadotJs().rpc.moon.isBlockFinalized(blockHash);
        expect(resp.isTrue, "Block finalization status mismatch").to.be.true;
      },
    });

    it({
      id: "T05",
      title: "should return as finalized when new block reorg happens",
      test: async function () {
        const blockHash = (await context.createBlock([], { finalize: false })).block.hash;
        await context.createBlock([], { finalize: false });
        await context.createBlock([], { finalize: true, parentHash: blockHash });

        const resp = await context.polkadotJs().rpc.moon.isBlockFinalized(blockHash);
        expect(resp.isTrue, "Block finalization status mismatch").to.be.true;
      },
    });

    it({
      id: "T06",
      title: "should return as finalized when true",
      test: async function () {
        await context.createBlock(
          await createRawTransaction(context, {
            to: BALTATHAR_ADDRESS,
            gas: 12_000_000n,
            gasPrice: BigInt(DEFAULT_TXN_MAX_BASE_FEE),
            value: 1_000_000n,
          }),
          { finalize: true }
        );

        const block = await context.viem("public").getBlock();
        const resp = await context
          .polkadotJs()
          .rpc.moon.isTxFinalized(block.transactions[0] as string);

        expect(resp.isTrue, "Transaction finalization status mismatch").to.be.true;
      },
    });

    it({
      id: "T07",
      title: "should return as unfinalized when false",
      test: async function () {
        await context.createBlock(
          await createRawTransaction(context, {
            to: BALTATHAR_ADDRESS,
            gas: 12_000_000n,
            gasPrice: BigInt(DEFAULT_TXN_MAX_BASE_FEE),
            value: 1_000_000n,
          }),
          { finalize: false }
        );

        const block = await context.viem("public").getBlock();
        const resp = await context
          .polkadotJs()
          .rpc.moon.isTxFinalized(block.transactions[0] as string);
        expect(resp.isFalse, "Transaction finalization status mismatch").to.be.true;
      },
    });

    it({
      id: "T08",
      title: "should return as unfinalized when txn not found",
      test: async function () {
        const txnHash = "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
        const resp = await context.polkadotJs().rpc.moon.isTxFinalized(txnHash);
        expect(resp.isFalse, "Transaction finalization status mismatch").to.be.true;
      },
    });

    it({
      id: "T09",
      title: "should return as finalized when new block is true",
      test: async function () {
        await context.createBlock(
          await createRawTransaction(context, {
            to: BALTATHAR_ADDRESS,
            gas: 12_000_000n,
            gasPrice: BigInt(DEFAULT_TXN_MAX_BASE_FEE),
            value: 1_000_000n,
          }),
          { finalize: false }
        );
        const block = await context.viem("public").getBlock();
        await context.createBlock([], { finalize: true });
        const resp = await context
          .polkadotJs()
          .rpc.moon.isTxFinalized(block.transactions[0] as string);
        expect(resp.isTrue, "Transaction finalization status mismatch").to.be.true;
      },
    });

    it({
      id: "T10",
      title: "should return as finalized when new block reorg happens",
      test: async function () {
        const blockHash = (
          await context.createBlock(
            await createRawTransaction(context, {
              to: BALTATHAR_ADDRESS,
              gas: 12_000_000n,
              gasPrice: BigInt(DEFAULT_TXN_MAX_BASE_FEE),
              value: 1_000_000n,
            }),
            { finalize: false }
          )
        ).block.hash;

        const block = await context.viem("public").getBlock();
        await context.createBlock([], { finalize: false });
        await context.createBlock([], { finalize: true, parentHash: blockHash });
        const resp = await context
          .polkadotJs()
          .rpc.moon.isTxFinalized(block.transactions[0] as string);
        expect(resp.isTrue, "Transaction finalization status mismatch").to.be.true;
      },
    });
  },
});

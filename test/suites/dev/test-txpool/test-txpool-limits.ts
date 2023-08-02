import "@moonbeam-network/api-augment";

import { describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  ALITH_PRIVATE_KEY,
  BALTATHAR_ADDRESS,
  BALTATHAR_PRIVATE_KEY,
  CHARLETH_PRIVATE_KEY,
  DOROTHY_PRIVATE_KEY,
  ETHAN_PRIVATE_KEY,
  FAITH_PRIVATE_KEY,
  GOLIATH_PRIVATE_KEY,
  createEthersTransaction,
  createRawTransfer,
  sendRawTransaction,
} from "@moonwall/util";
import { create } from "domain";
import { afterEach } from "node:test";
import { start } from "repl";
import { encodeDeployData } from "viem";

describeSuite({
  id: "D3303",
  title: "TxPool - Limits",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {

    afterEach(async () => {
      //Drain pool of all txns
      while (true) {
        await context.createBlock();
        if ((await context.viem().getBlock()).transactions.length == 0) {
          break;
        }
      }
    });

    it({
      id: "T01",
      title: "should be able to fill a block with 260 tx",
      test: async function () {
        for (let i = 0; i < 800; i++) {
          const rawTxn = await createRawTransfer(context, BALTATHAR_ADDRESS, 1n, {
            nonce: i,
            privateKey: ALITH_PRIVATE_KEY,
          });
          await sendRawTransaction(context, rawTxn);
        }

        await context.createBlock();
        expect((await context.viem().getBlock()).transactions.length).toBe(714);
      },
    });

    it({
      id: "T02",
      title: "should be able to fill a block with 64 contract creations tx",
      test: async function () {
        const { abi, bytecode } = fetchCompiledContract("MultiplyBy7");
        const deployData = encodeDeployData({
          abi,
          bytecode,
        });

        for (let i = 0; i < 120; i++) {
          const rawTxn = await createEthersTransaction(context, {
            data: deployData,
            nonce: i,
            gas: 400000n,
            privateKey: BALTATHAR_PRIVATE_KEY,
          });
          await sendRawTransaction(context, rawTxn);
        }

        await context.createBlock();
        expect((await context.viem().getBlock()).transactions.length).toBe(95);
      },
    });

    // // 8192 is the number of tx that can be sent to the Pool
    // // before it throws an error and drops all tx
    it({
      id: "T03",
      title:
        "should be able to send 8192 tx to the pool " +
        "and have them all published within the following blocks",
      test: async function () {
        for (let i = 0; i < 8192; i++) {
          // for (let i = 0; i < 8192; i++) {
          const rawTxn = await createRawTransfer(context, BALTATHAR_ADDRESS, 1n, {
            nonce: i,
            gas: 400000n,
            privateKey: CHARLETH_PRIVATE_KEY,
          });
          await sendRawTransaction(context, rawTxn);
        }
        const inspectBlob = (await context
          .viem()
          .transport.request({ method: "txpool_inspect" })) as any;

        const txPoolSize = Object.keys(inspectBlob.pending[ALITH_ADDRESS.toLowerCase()]).length;

        expect(txPoolSize).toBe(8192);

        let blocks = 1;
        while (true) {
          await context.createBlock();

          const inspectBlob = (await context
            .viem()
            .transport.request({ method: "txpool_inspect" })) as any;
          const txPoolSize = Object.keys(
            inspectBlob.pending[ALITH_ADDRESS.toLowerCase()] || {}
          ).length;
          log(`Transactions left in pool: ${txPoolSize}`);

          if ((await context.viem().getBlock()).transactions.length == 0) {
            break;
          }
          blocks++;
        }
        log(`Transaction pool was emptied in ${blocks} blocks.`);
      },
    });

    it({
      id: "T04",
      title: "shouldn't work for 8193",
      test: async function () {
        try {
          for (let i = 0; i < 8193; i++) {
            const rawTxn = await createRawTransfer(context, BALTATHAR_ADDRESS, 1n, {
              nonce: i,
              gas: 400000n,
              privateKey: DOROTHY_PRIVATE_KEY,
            });
            await sendRawTransaction(context, rawTxn);
          }
        } catch (e: any) {
          expect(e.message).toContain("submit transaction to pool failed: Ok(ImmediatelyDropped)");
        }

        const inspectBlob = (await context
          .viem()
          .transport.request({ method: "txpool_inspect" })) as any;

        expect(inspectBlob).toMatchObject({
          pending: {},
          queued: {},
        });
      },
    });

    it({
      id: "T05",
      title:
        "should be able to send 8192 tx to the pool and have them" +
        " all published within the following blocks - bigger tx",
      timeout: 400_000,
      test: async function () {
        const { abi, bytecode } = fetchCompiledContract("MultiplyBy7");
        const deployData = encodeDeployData({
          abi,
          bytecode,
        });

        for (let i = 0; i < 8192; i++) {
          const rawTxn = await createEthersTransaction(context, {
            data: deployData,
            nonce: i,
            gas: 400000n,
            privateKey: ETHAN_PRIVATE_KEY,
          });
          await sendRawTransaction(context, rawTxn);
        }
        const inspectBlob = (await context
          .viem()
          .transport.request({ method: "txpool_inspect" })) as any;

        const txPoolSize = Object.keys(inspectBlob.pending[ALITH_ADDRESS.toLowerCase()]).length;

        expect(txPoolSize).toBe(8192);

        let blocks = 1;
        while (true) {
          await context.createBlock();

          const inspectBlob = (await context
            .viem()
            .transport.request({ method: "txpool_inspect" })) as any;
          const txPoolSize = Object.keys(
            inspectBlob.pending[ALITH_ADDRESS.toLowerCase()] || {}
          ).length;
          log(`Transactions left in pool: ${txPoolSize}`);

          if ((await context.viem().getBlock()).transactions.length == 0) {
            break;
          }
          blocks++;
        }
        log(`Transaction pool was emptied in ${blocks} blocks.`);
      },
    });

    it({
      id: "T06",
      title:
        "should be able to send 8192 tx to the pool and have them" +
        " all published within the following blocks - bigger tx",
      timeout: 400_000,
      test: async function () {
        const { abi, bytecode } = fetchCompiledContract("MultiplyBy7");
        const deployData = encodeDeployData({
          abi,
          bytecode,
        });

        for (let i = 0; i < 8192; i++) {
          const rawTxn = await createEthersTransaction(context, {
            data: deployData,
            nonce: i,
            gas: 400000n,
            privateKey: FAITH_PRIVATE_KEY,
          });
          await sendRawTransaction(context, rawTxn);
        }
        const inspectBlob = (await context
          .viem()
          .transport.request({ method: "txpool_inspect" })) as any;

        const txPoolSize = Object.keys(inspectBlob.pending[ALITH_ADDRESS.toLowerCase()]).length;

        expect(txPoolSize).toBe(8192);

        let blocks = 1;
        while (true) {
          await context.createBlock();

          const inspectBlob = (await context
            .viem()
            .transport.request({ method: "txpool_inspect" })) as any;
          const txPoolSize = Object.keys(
            inspectBlob.pending[ALITH_ADDRESS.toLowerCase()] || {}
          ).length;
          log(`Transactions left in pool: ${txPoolSize}`);

          if ((await context.viem().getBlock()).transactions.length == 0) {
            break;
          }
          blocks++;
        }
        log(`Transaction pool was emptied in ${blocks} blocks.`);
      },
    });

    it({
      id: "T07",
      title: "shouldn't work for 8193 - bigger tx",
      timeout: 400_000,
      test: async function () {
        const { abi, bytecode } = fetchCompiledContract("MultiplyBy7");
        const deployData = encodeDeployData({
          abi,
          bytecode,
        });
        try {
          for (let i = 0; i < 8192; i++) {
            const rawTxn = await createEthersTransaction(context, {
              data: deployData,
              nonce: i,
              gas: 400000n,
              privateKey: GOLIATH_PRIVATE_KEY,
            });
            await sendRawTransaction(context, rawTxn);
          }
        } catch (e: any) {
          expect(e.message).toContain("submit transaction to pool failed: Ok(ImmediatelyDropped)");
        }

        const inspectBlob = (await context
          .viem()
          .transport.request({ method: "txpool_inspect" })) as any;

        expect(inspectBlob).toMatchObject({
          pending: {},
          queued: {},
        });
      },
    });
  },
});

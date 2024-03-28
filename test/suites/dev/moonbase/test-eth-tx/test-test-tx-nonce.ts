import "@moonbeam-network/api-augment";
import { customDevRpcRequest, describeSuite, expect } from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  BALTATHAR_ADDRESS,
  CHARLETH_ADDRESS,
  createRawTransfer,
} from "@moonwall/util";

describeSuite({
  id: "D011303",
  title: "Ethereum Transaction - Nonce",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should be at 0 before using it",
      test: async function () {
        expect(await context.viem().getTransactionCount({ address: BALTATHAR_ADDRESS })).toBe(0);
      },
    });

    it({
      id: "T02",
      title: "should be at 0 for genesis account",
      test: async function () {
        expect(await context.viem().getTransactionCount({ address: ALITH_ADDRESS })).toBe(0);
      },
    });

    it({
      id: "T03",
      title: "should stay at 0 before block is created",
      test: async function () {
        await customDevRpcRequest("eth_sendRawTransaction", [
          await createRawTransfer(context, ALITH_ADDRESS, 512),
        ]);

        expect(await context.viem().getTransactionCount({ address: ALITH_ADDRESS })).toBe(0);
        await context.createBlock();
      },
    });

    it({
      id: "T04",
      title: "should stay at previous before block is created",
      test: async function () {
        const blockNumber = await context.viem().getBlockNumber();
        const nonce = await context.viem().getTransactionCount({ address: ALITH_ADDRESS });
        await context.createBlock(await createRawTransfer(context, ALITH_ADDRESS, 512));

        expect(
          await context.viem().getTransactionCount({ address: ALITH_ADDRESS, blockNumber })
        ).toBe(nonce);
      },
    });

    it({
      id: "T05",
      title: "pending transaction nonce",
      test: async function () {
        const blockNumber = await context.viem().getBlockNumber();
        const nonce = await context.viem().getTransactionCount({ address: ALITH_ADDRESS });

        await customDevRpcRequest("eth_sendRawTransaction", [
          await createRawTransfer(context, CHARLETH_ADDRESS, 512),
        ]);

        expect(
          await context.viem().getTransactionCount({ address: ALITH_ADDRESS }),
          "should not increase transaction count"
        ).toBe(nonce);
        expect(
          await context
            .viem("public")
            .getTransactionCount({ address: ALITH_ADDRESS, blockTag: "latest" }),
          "should not increase transaction count in latest block"
        ).toBe(nonce);
        expect(
          await context
            .viem("public")
            .getTransactionCount({ address: ALITH_ADDRESS, blockTag: "pending" }),
          "should increase transaction count in pending block"
        ).toBe(nonce + 1);
        await context.createBlock();
      },
    });

    it({
      id: "T06",
      title: "transferring Nonce",
      test: async function () {
        const nonce = await context.viem().getTransactionCount({ address: ALITH_ADDRESS });

        await context.createBlock([await createRawTransfer(context, BALTATHAR_ADDRESS, 512)]);

        expect(
          await context.viem().getTransactionCount({ address: ALITH_ADDRESS }),
          "should increase the sender nonce"
        ).toBe(nonce + 1);
        expect(
          await context.viem().getTransactionCount({ address: BALTATHAR_ADDRESS }),
          "should not increase the receiver nonce"
        ).toBe(0);
        await context.createBlock();
      },
    });
  },
});

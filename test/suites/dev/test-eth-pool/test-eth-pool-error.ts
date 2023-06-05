import { describeSuite, expect } from "@moonwall/cli";
import {
  ALITH_GENESIS_TRANSFERABLE_BALANCE,
  BALTATHAR_ADDRESS,
  BALTATHAR_PRIVATE_KEY,
  CHARLETH_ADDRESS,
  CHARLETH_PRIVATE_KEY,
  MIN_GAS_PRICE,
  createRawTransfer,
  sendRawTransaction,
} from "@moonwall/util";
import { customDevRpcRequest } from "../../../helpers/common.js";
import { parseGwei } from "viem";

describeSuite({
  id: "D1102",
  title: "Ethereum Rpc pool errors",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "already known #1",
      test: async function () {
        const tx = (await createRawTransfer(context, BALTATHAR_ADDRESS, 1)) as `0x${string}`;
        await sendRawTransaction(context, tx);

        expect(async () => await sendRawTransaction(context, tx)).rejects.toThrowError(
          "already known"
        );
        await context.createBlock();
      },
    });

    it({
      id: "T02",
      title: "replacement transaction underpriced",
      test: async function () {
        const tx1 = await createRawTransfer(context, CHARLETH_ADDRESS, 1, {
          nonce: 0,
          gasPrice: parseGwei("15"),
          privateKey: BALTATHAR_PRIVATE_KEY,
          type: "legacy"
        });
        const tx2 = await createRawTransfer(context, BALTATHAR_ADDRESS, 2, {
          nonce: 0,
          gasPrice: parseGwei("10"),
          privateKey: BALTATHAR_PRIVATE_KEY,
          type: "legacy"
        });
        await customDevRpcRequest("eth_sendRawTransaction", [tx1]);
        expect(
          async () => await customDevRpcRequest("eth_sendRawTransaction", [tx2])
        ).rejects.toThrowError("replacement transaction underpriced");

        await context.createBlock();
      },
    });

    it({
      id: "T03",
      title: "nonce too low",
      test: async function () {
        const nonce = await context
          .viemClient("public")
          .getTransactionCount({ address: CHARLETH_ADDRESS });
        const tx1 = await createRawTransfer(context, BALTATHAR_ADDRESS, 1, {
          nonce,
          privateKey: CHARLETH_PRIVATE_KEY,
        });
        await context.createBlock(tx1);

        const tx2 = await createRawTransfer(context, CHARLETH_ADDRESS, 2, {
          nonce,
          privateKey: CHARLETH_PRIVATE_KEY,
        });
        expect(
          async () => await customDevRpcRequest("eth_sendRawTransaction", [tx2])
        ).rejects.toThrowError("nonce too low");
        await context.createBlock();
      },
    });

    it({
      id: "T04",
      title: "already known #2",
      test: async function () {
        const tx1 = await createRawTransfer(context, BALTATHAR_ADDRESS, 1, {
          nonce: 0,
          gasPrice: MIN_GAS_PRICE,
        });
        await context.createBlock(tx1);
        const tx2 = await createRawTransfer(context, BALTATHAR_ADDRESS, 1, {
          nonce: 0,
          gasPrice: MIN_GAS_PRICE,
        });
        expect(
          async () => await customDevRpcRequest("eth_sendRawTransaction", [tx1])
        ).rejects.toThrowError("already known");
      },
    });

    it({
      id: "T05",
      title: "insufficient funds for gas * price + value",
      test: async function () {
        const ZEROED_PKEY = "0xbf2a9f29a7631116a1128e34fcf8817581fb3ec159ef2be004b459bc33f2ed2d";
        const tx = await createRawTransfer(context, BALTATHAR_ADDRESS, 1, {
          privateKey: ZEROED_PKEY,
        });

        expect(
          async () => await customDevRpcRequest("eth_sendRawTransaction", [tx])
        ).rejects.toThrowError("insufficient funds for gas * price + value");
      },
    });

    it({
      id: "T06",
      title: "exceeds block gas limit",
      test: async function () {
        const tx = await createRawTransfer(context, BALTATHAR_ADDRESS, 1, {
          gas: 1_000_000_0000n,
        });

        expect(
          async () => await customDevRpcRequest("eth_sendRawTransaction", [tx])
        ).rejects.toThrowError("exceeds block gas limit");
      },
    });

    it({
      id: "T07",
      title: "insufficient funds for gas * price + value",
      test: async function () {
        const amount = ALITH_GENESIS_TRANSFERABLE_BALANCE - 21000n * 10_000_000_000n + 1n;
        const tx = await createRawTransfer(context, BALTATHAR_ADDRESS, amount);

        expect(
          async () => await customDevRpcRequest("eth_sendRawTransaction", [tx])
        ).rejects.toThrowError("insufficient funds for gas * price + value");
      },
    });

    it({
      id: "T08",
      title: "max priority fee per gas higher than max fee per gast",
      modifier: "skip", // client libraries block invalid txns like this
      test: async function () {
        const tx = await createRawTransfer(context, BALTATHAR_ADDRESS, 1n, {
          maxFeePerGas: 100_000_000_000n,
          maxPriorityFeePerGas: 200_000_000_000n,
        });

        expect(
          async () => await customDevRpcRequest("eth_sendRawTransaction", [tx])
        ).rejects.toThrowError("max priority fee per gas higher than max fee per gas");
      },
    });
  },
});

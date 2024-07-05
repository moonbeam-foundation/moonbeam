import "@moonbeam-network/api-augment";
import { describeSuite } from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  ALITH_PRIVATE_KEY,
  BALTATHAR_ADDRESS,
  BALTATHAR_PRIVATE_KEY,
  createRawTransfer,
  sendRawTransaction,
} from "@moonwall/util";
import { ethers } from "ethers";

describeSuite({
  id: "D013904",
  title: "TxPool - Limits",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should be able to fill a block with 582 tx",
      test: async function () {
        console.time("1st-balance");
        const balanceTest = await context.viem().getBalance({ address: BALTATHAR_ADDRESS });
        console.timeEnd("1st-balance");
        // TODO: test how many transactions can fit in the block

        console.time("injecting-operations");
        const nonce = {
          [BALTATHAR_ADDRESS]: 0,
          [ALITH_ADDRESS]: 0,
        };
        for (let i = 0; i < 8192; i++) {
          const address = i % 2 == 0 ? BALTATHAR_ADDRESS : ALITH_ADDRESS;
          const rawTxn = await createRawTransfer(context, address, i + 1, {
            nonce: i,
            privateKey: true ? BALTATHAR_PRIVATE_KEY : ALITH_PRIVATE_KEY,
          });
          await sendRawTransaction(context, rawTxn);
          nonce[address] += 1;
        }
        console.timeEnd("injecting-operations");

        console.time("2nd-fetch-balance");
        await context.viem().getBalance({ address: BALTATHAR_ADDRESS });
        console.timeEnd("2nd-fetch-balance");

        for (let i = 0; i < 1000; i++) {
          (context.ethers().provider as ethers.JsonRpcProvider)
            .send("eth_getTransactionCount", [BALTATHAR_ADDRESS, "pending"])
            .catch((e) => console.log(e));
        }
        await new Promise((r) => setTimeout(r, 120000));
        return;
        for (let i = 0; i < 100; i++) {
          for (let i = 0; i < 1000; i++) {
            (context.ethers().provider as ethers.JsonRpcProvider)
              .send("eth_getTransactionCount", [BALTATHAR_ADDRESS, "pending"])
              .catch((e) => console.log(e));
            await new Promise((r) => setTimeout(r, 10));
          }
          const result = await (context.ethers().provider as ethers.JsonRpcProvider).send(
            "eth_getBalance",
            [BALTATHAR_ADDRESS, "latest"]
          );
          await new Promise((r) => setTimeout(r, 100));

          console.time(`fetch balance (iter: ${i + 1})`);
          await context.viem().getBalance({ address: BALTATHAR_ADDRESS });
          console.timeEnd(`fetch balance (iter: ${i + 1})`);
        }

        //console.time("pool");
        //const inspectBlob = (await context
        //  .viem()
        //  .transport.request({ method: "txpool_inspect" })) as any;
        //console.log(inspectBlob.pending)
        //const txPoolSize = Object.keys(inspectBlob.pending[ALITH_ADDRESS.toLowerCase()]).length;
        //console.log(txPoolSize)
        //console.timeEnd("pool");

        console.time("get-pending-ops");
        console.log((await context.polkadotJs().rpc.author.pendingExtrinsics()).length);
        console.timeEnd("get-pending-ops");

        console.time("last-balance");
        await context.viem().getBalance({ address: BALTATHAR_ADDRESS });
        console.timeEnd("last-balance");
      },
    });
  },
});

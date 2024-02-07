import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, BALTATHAR_ADDRESS, createRawTransfer } from "@moonwall/util";
import { PublicClient, createPublicClient, webSocket } from "viem";

describeSuite({
  id: "D3005",
  title: "Subscription - Block headers",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let client: PublicClient;

    beforeAll(async () => {
      const transport = webSocket(context.viem().transport.url.replace("http", "ws"));
      client = createPublicClient({
        transport,
      });
    });

    it({
      id: "T01",
      title: "should return a valid subscriptionId",
      test: async function () {
        const result = (await client.transport.request({
          method: "eth_subscribe",
          params: ["newHeads"],
        })) as any;

        expect(result.length).toBe(34);
      },
    });

    it({
      id: "T02",
      title: "should send notification on new block",
      test: async function () {
        const blocks: any[] = [];
        const unwatch = client.watchBlocks({
          onBlock: (block) => blocks.push(block),
        });

        await context.createBlock(createRawTransfer(context, BALTATHAR_ADDRESS, 0));
        unwatch();

        expect(blocks[0]).to.include({
          author: ALITH_ADDRESS.toLowerCase(),
          difficulty: 0n,
          extraData: "0x",
          logsBloom: `0x${"0".repeat(512)}`,
          miner: ALITH_ADDRESS.toLowerCase(),
          receiptsRoot: "0xf78dfb743fbd92ade140711c8bbc542b5e307f0ab7984eff35d751969fe57efa",
          sha3Uncles: "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
          transactionsRoot: "0x199e8cd01da80b37a1dc061d83f5845334ac861e0c839636f1cad280f5a9bd49",
        });
        expect(blocks[0].nonce).to.be.eq("0x0000000000000000");
      },
    });
  },
});

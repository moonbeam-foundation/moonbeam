import "@moonbeam-network/api-augment";
import { describeSuite, expect, beforeAll } from "@moonwall/cli";

describeSuite({
  id: "D023910",
  title: "TxPool - New block",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      await context.deployContract!("MultiplyBy7");
      await context.createBlock();
    });

    it({
      id: "T01",
      title: "should reset the txpool",
      test: async function () {
        const inspect = (await context
          .viem()
          .transport.request({ method: "txpool_inspect" })) as any;
        expect(inspect.pending).to.be.empty;
        const content = (await context
          .viem()
          .transport.request({ method: "txpool_inspect" })) as any;
        expect(content.pending).to.be.empty;
      },
    });
  },
});

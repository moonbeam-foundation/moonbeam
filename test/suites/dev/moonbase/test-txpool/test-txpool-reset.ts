import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";

describeSuite({
  id: "D013909",
  title: "TxPool - Genesis",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should be empty",
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

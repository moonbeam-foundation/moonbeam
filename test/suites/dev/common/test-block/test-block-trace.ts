import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS } from "@moonwall/util";

describeSuite({
  id: "D010406",
  title: "Block Trace - Substrate",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let blockHash: string;

    beforeAll(async () => {
      // Create a block with a substrate transaction
      const { block } = await context.createBlock(
        context.polkadotJs().tx.balances.transferAllowDeath(ALITH_ADDRESS, 1000000000000)
      );
      blockHash = block.hash.toString();
      console.log(blockHash);
    });

    it({
      id: "T01",
      title: "should trace block with substrate transactions",
      test: async function () {
        // Get the block trace
        const trace = await context.polkadotJs().rpc.state.traceBlock(blockHash, null, null, null);

        // Verify the trace was successful
        expect(trace).to.not.be.null;
        expect(trace).to.not.be.undefined;
      },
    });
  },
});

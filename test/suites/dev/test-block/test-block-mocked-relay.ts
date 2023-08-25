import "@moonbeam-network/api-augment";
import { expect, describeSuite, beforeAll } from "@moonwall/cli";
import { CumulusPrimitivesParachainInherentParachainInherentData } from "@polkadot/types/lookup";

describeSuite({
  id: "D0405",
  title: "Block - Mocked relaychain block",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      expect(await context.viem().getBlockNumber()).toBe(0n);
    });

    it({
      id: "T01",
      title: "should contain block details",
      test: async function () {
        const blockResult = await context.createBlock();
        const blockData = await context.polkadotJs().rpc.chain.getBlock(blockResult.block.hash);
        expect(
          (
            blockData.block.extrinsics[1].method
              .args[0] as CumulusPrimitivesParachainInherentParachainInherentData
          ).validationData.relayParentNumber.toString()
        ).to.eq("1000");
        const blockResult2 = await context.createBlock();
        const blockData2 = await context.polkadotJs().rpc.chain.getBlock(blockResult2.block.hash);
        expect(
          (
            blockData2.block.extrinsics[1].method
              .args[0] as CumulusPrimitivesParachainInherentParachainInherentData
          ).validationData.relayParentNumber.toString()
        ).to.eq("1002");
      },
    });
  },
});

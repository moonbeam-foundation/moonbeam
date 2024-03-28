import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { FAITH_ADDRESS, FAITH_PRIVATE_KEY, getBlockExtrinsic } from "@moonwall/util";
import { setAuthorMappingKeysViaPrecompile } from "../../../../helpers";

describeSuite({
  id: "D012817",
  title: "Precompile Author Mapping - Set Faith mapping with 0 keys",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should revert",
      test: async function () {
        await setAuthorMappingKeysViaPrecompile(
          context,
          FAITH_ADDRESS,
          FAITH_PRIVATE_KEY,
          "0x",
          true
        );
        const { extrinsic, events, resultEvent } = await getBlockExtrinsic(
          context.polkadotJs(),
          await context.polkadotJs().rpc.chain.getBlockHash(),
          "ethereum",
          "transact"
        );

        expect(extrinsic).to.exist;
        expect(resultEvent?.method).to.equal("ExtrinsicSuccess");
        expect(
          (events.find((e) => e.section == "ethereum" && e.method == "Executed")?.data[3] as any)
            .isRevert
        ).to.be.true;
      },
    });
  },
});

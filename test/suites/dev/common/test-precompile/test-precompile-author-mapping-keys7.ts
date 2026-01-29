import "@moonbeam-network/api-augment";
import {
  FAITH_ADDRESS,
  FAITH_PRIVATE_KEY,
  describeSuite,
  expect,
  getBlockExtrinsic,
} from "moonwall";
import { originalKeys, setAuthorMappingKeysViaPrecompile } from "../../../../helpers";

describeSuite({
  id: "D010407",
  title: "Precompile Author Mapping - Set Faith only 1 key",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    it({
      id: "T01",
      title: "should revert",
      test: async function () {
        await setAuthorMappingKeysViaPrecompile(
          context,
          FAITH_ADDRESS,
          FAITH_PRIVATE_KEY,
          originalKeys[0],
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
          (events.find((e) => e.section === "ethereum" && e.method === "Executed")?.data[3] as any)
            .isRevert
        ).to.be.true;
      },
    });
  },
});

import "@moonbeam-network/api-augment";
import { describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import {
  FAITH_PRIVATE_KEY,
  PRECOMPILE_AUTHOR_MAPPING_ADDRESS,
  createViemTransaction,
  getBlockExtrinsic,
} from "@moonwall/util";
import { encodeFunctionData } from "viem";

describeSuite({
  id: "D012814",
  title: "Precompile Author Mapping - Removing non-existing author",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should revert",
      test: async function () {
        const { abi } = fetchCompiledContract("AuthorMapping");

        await context.createBlock(
          createViemTransaction(context, {
            to: PRECOMPILE_AUTHOR_MAPPING_ADDRESS,
            privateKey: FAITH_PRIVATE_KEY,
            data: encodeFunctionData({ abi, functionName: "removeKeys" }),
            skipEstimation: true,
          })
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

    it({
      id: "T02",
      title: "should not send KeysRemoved event",
      test: async function () {
        const { events } = await getBlockExtrinsic(
          context.polkadotJs(),
          await context.polkadotJs().rpc.chain.getBlockHash(),
          "ethereum",
          "transact"
        );
        expect(events.find((e) => e.section === "authorMapping" && e.method === "KeysRemoved")).to
          .not.exist;
      },
    });
  },
});

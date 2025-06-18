import "@moonbeam-network/api-augment";
import { expect, describeSuite, beforeAll } from "@moonwall/cli";
import { dorothy, getBlockExtrinsic } from "@moonwall/util";
import type { ApiPromise } from "@polkadot/api";

// Keys used to set author-mapping in the tests
const originalKeys = [
  "0x0000000000000000000000000000000000000000000000000000000000000001",
  "0x0000000000000000000000000000000000000000000000000000000000000002",
];
// Concatenated keys
const concatOriginalKeys = `0x${originalKeys.map((key) => key.slice(2)).join("")}`;

describeSuite({
  id: "D020209",
  title: "Author Mapping - Removing non-existing author",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let api: ApiPromise;

    beforeAll(async function () {
      // Remove the keys
      api = context.polkadotJs();
      await api.tx.authorMapping.removeKeys().signAndSend(dorothy);
      await context.createBlock();
    });

    it({
      id: "T01",
      title: "should fail",
      test: async function () {
        const { extrinsic, resultEvent } = await getBlockExtrinsic(
          api,
          await api.rpc.chain.getBlockHash(),
          "authorMapping",
          "removeKeys"
        );

        expect(extrinsic).to.exist;
        expect(resultEvent?.method).to.equal("ExtrinsicFailed");
      },
    });

    it({
      id: "T02",
      title: "should not send KeysRemoved event",
      test: async function () {
        const { events } = await getBlockExtrinsic(
          api,
          await api.rpc.chain.getBlockHash(),
          "authorMapping",
          "removeKeys"
        );
        expect(events.find((e) => e.section === "authorMapping" && e.method === "KeysRemoved")).to
          .not.exist;
      },
    });
  },
});

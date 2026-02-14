import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, dorothy, expect, getBlockExtrinsic } from "moonwall";
import type { ApiPromise } from "@polkadot/api";

describeSuite({
  id: "D020209",
  title: "Author Mapping - Removing non-existing author",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
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

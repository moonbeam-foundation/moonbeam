import "@moonbeam-network/api-augment";
import { expect, describeSuite, beforeAll } from "@moonwall/cli";
import { charleth, getBlockExtrinsic } from "@moonwall/util";
import type { ApiPromise } from "@polkadot/api";

// Keys used to set author-mapping in the tests
const originalKeys = [
  "0x0000000000000000000000000000000000000000000000000000000000000001",
  "0x0000000000000000000000000000000000000000000000000000000000000002",
];
// Concatenated keys
const concatOriginalKeys = `0x${originalKeys.map((key) => key.slice(2)).join("")}`;

describeSuite({
  id: "D020210",
  title: "Author Mapping - Remove Charlie keys",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let api: ApiPromise;

    beforeAll(async function () {
      api = context.polkadotJs();
      await (api.tx.authorMapping.setKeys as any)(concatOriginalKeys).signAndSend(charleth);
      await context.createBlock();

      // Remove the keys
      await api.tx.authorMapping.removeKeys().signAndSend(charleth);
      await context.createBlock();
    });

    it({
      id: "T01",
      title: "should succeed",
      test: async function () {
        const { extrinsic, resultEvent } = await getBlockExtrinsic(
          api,
          await api.rpc.chain.getBlockHash(),
          "authorMapping",
          "removeKeys"
        );

        expect(extrinsic).to.exist;
        expect(resultEvent?.method).to.equal("ExtrinsicSuccess");
      },
    });

    it({
      id: "T02",
      title: "should send KeysRemoved event",
      test: async function () {
        const { events } = await getBlockExtrinsic(
          api,
          await api.rpc.chain.getBlockHash(),
          "authorMapping",
          "removeKeys"
        );
        expect(events.find((e) => e.section === "authorMapping" && e.method === "KeysRemoved")).to
          .exist;
      },
    });

    it({
      id: "T03",
      title: "should remove keys",
      test: async function () {
        const mapping = await api.query.authorMapping.mappingWithDeposit(originalKeys[0]);
        expect(mapping.isNone).to.be.true;
      },
    });

    it({
      id: "T04",
      title: "should remove nimbus mapping",
      test: async function () {
        const nimbusLookup = (await api.query.authorMapping.nimbusLookup(charleth.address)) as any;
        expect(nimbusLookup.isNone).to.be.true;
      },
    });
  },
});

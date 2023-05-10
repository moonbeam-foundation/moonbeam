import "@moonbeam-network/api-augment";
import { expect, describeSuite, beforeAll } from "@moonwall/cli";
import { charleth, dorothy, getBlockExtrinsic } from "@moonwall/util";
import { ApiPromise } from "@polkadot/api";

// Keys used to set author-mapping in the tests
const originalKeys = [
  "0x0000000000000000000000000000000000000000000000000000000000000001",
  "0x0000000000000000000000000000000000000000000000000000000000000002",
];
// Concatenated keys
const concatOriginalKeys = `0x${originalKeys.map((key) => key.slice(2)).join("")}`;

describeSuite({
  id: "D216",
  title: "Author Mapping - Update someone else nimbus key",
  foundationMethods: "dev",
  testCases: ({ context, log, it }) => {
    let api: ApiPromise;

    beforeAll(async function () {
      api = context.polkadotJs({ type: "moon" });
      await (api.tx.authorMapping.setKeys as any)(concatOriginalKeys).signAndSend(charleth);
      await context.createBlock();

      // Setting same key but with ethan
      await (api.tx.authorMapping.setKeys as any)(concatOriginalKeys).signAndSend(dorothy);
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
          "setKeys"
        );

        expect(extrinsic).to.exist;
        expect(resultEvent.method).to.equal("ExtrinsicFailed");
      },
    });

    it({
      id: "T02",
      title: "should not send any authorMapping event",
      test: async function () {
        const { events } = await getBlockExtrinsic(
          api,
          await api.rpc.chain.getBlockHash(),
          "authorMapping",
          "removeKeys"
        );
        expect(events.find((e) => e.section == "authorMapping")).to.not.exist;
      },
    });

    it({
      id: "T03",
      title: "should keep the same keys to Faith",
      test: async function () {
        const mapping = await api.query.authorMapping.mappingWithDeposit(originalKeys[0]);
        expect(mapping.isSome).to.be.true;
        expect(mapping.unwrap().account.toString()).to.equal(charleth.address);
        expect(mapping.unwrap().keys_.toString()).to.equal(originalKeys[1]);
      },
    });

    it({
      id: "T04",
      title: "should not set nimbus lookup to Ethan",
      test: async function () {
        const nimbusLookup = (await api.query.authorMapping.nimbusLookup(dorothy.address)) as any;
        expect(nimbusLookup.isNone).to.be.true;
      },
    });

    it({
      id: "T05",
      title: "should keep the same nimbus lookup to Faith",
      test: async function () {
        const nimbusLookup = (await api.query.authorMapping.nimbusLookup(charleth.address)) as any;
        expect(nimbusLookup.isSome).to.be.true;
        expect(nimbusLookup.unwrap().toString()).to.equal(originalKeys[0]);
      },
    });
  },
});

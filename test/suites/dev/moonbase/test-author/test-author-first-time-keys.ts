import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
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
  id: "D020203",
  title: "Author Mapping - Set Charlie first time keys",
  foundationMethods: "dev",
  testCases: ({ context, log, it }) => {
    let api: ApiPromise;

    beforeAll(async function () {
      api = context.polkadotJs();
      log(`Setting account ${charleth.address} keys: ${concatOriginalKeys}`);
      // TODO: fix all setKeys with api 1600.1
      await api.tx.authorMapping.setKeys(concatOriginalKeys).signAndSend(charleth);
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
          "setKeys"
        );

        expect(extrinsic).to.exist;
        expect(resultEvent!.method).to.equal("ExtrinsicSuccess");
      },
    });

    it({
      id: "T02",
      title: "should send KeysRegistered event",
      test: async function () {
        const { events } = await getBlockExtrinsic(
          api,
          await api.rpc.chain.getBlockHash(),
          "authorMapping",
          "setKeys"
        );
        expect(events.find((e) => e.section === "authorMapping" && e.method === "KeysRegistered"))
          .to.exist;
      },
    });

    it({
      id: "T03",
      title: "should set new keys",
      test: async function () {
        const mapping = await api.query.authorMapping.mappingWithDeposit(originalKeys[0]);
        expect(mapping.isSome).to.be.true;
        expect(mapping.unwrap().account.toString()).to.equal(charleth.address);
        expect(mapping.unwrap().keys_.toString()).to.equal(originalKeys[1]);
      },
    });

    it({
      id: "T04",
      title: "should set correct nimbus lookup",
      test: async function () {
        const nimbusLookup = (await api.query.authorMapping.nimbusLookup(charleth.address)) as any;
        expect(nimbusLookup.isSome).to.be.true;
        expect(nimbusLookup.unwrap().toString()).to.equal(originalKeys[0]);
      },
    });
  },
});

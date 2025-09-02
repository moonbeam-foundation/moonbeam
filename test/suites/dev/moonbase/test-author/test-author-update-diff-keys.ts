import "@moonbeam-network/api-augment";
import { expect, describeSuite, beforeAll } from "@moonwall/cli";
import { CHARLETH_ADDRESS, charleth, getBlockExtrinsic } from "@moonwall/util";
import type { ApiPromise } from "@polkadot/api";

// Keys used to set author-mapping in the tests
const originalKeys = [
  "0x0000000000000000000000000000000000000000000000000000000000000001",
  "0x0000000000000000000000000000000000000000000000000000000000000002",
];
// Concatenated keys
const concatOriginalKeys = `0x${originalKeys.map((key) => key.slice(2)).join("")}`;

describeSuite({
  id: "D020215",
  title: "Author Mapping - Update different keys",
  foundationMethods: "dev",
  testCases: ({ context, log, it }) => {
    const newKeys = [
      "0x0000000000000000000000000000000000000000000000000000000000000003",
      "0x0000000000000000000000000000000000000000000000000000000000000004",
    ];
    const concatNewKeys = `0x${newKeys.map((key) => key.slice(2)).join("")}`;
    let api: ApiPromise;

    beforeAll(async function () {
      api = context.polkadotJs();
      await api.tx.authorMapping.setKeys(concatOriginalKeys).signAndSend(charleth);
      await context.createBlock();

      // Updating with different keys
      await api.tx.authorMapping.setKeys(concatNewKeys).signAndSend(charleth);
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
        expect(resultEvent?.method).to.equal("ExtrinsicSuccess");
      },
    });

    it({
      id: "T02",
      title: "should send KeysRotated event",
      test: async function () {
        const { events } = await getBlockExtrinsic(
          api,
          await api.rpc.chain.getBlockHash(),
          "authorMapping",
          "setKeys"
        );
        expect(events.find((e) => e.section === "authorMapping" && e.method === "KeysRotated")).to
          .exist;
      },
    });

    it({
      id: "T03",
      title: "should remove previous keys",
      test: async function () {
        const mapping = await api.query.authorMapping.mappingWithDeposit(originalKeys[0]);
        expect(mapping.isNone).to.be.true;
      },
    });

    it({
      id: "T04",
      title: "should set new keys",
      test: async function () {
        const mapping = await api.query.authorMapping.mappingWithDeposit(newKeys[0]);
        expect(mapping.isSome).to.be.true;
        expect(mapping.unwrap().account.toString()).to.equal(charleth.address);
        expect(mapping.unwrap().keys_.toString()).to.equal(newKeys[1]);
      },
    });

    it({
      id: "T05",
      title: "should set correct nimbus lookup",
      test: async function () {
        const nimbusLookup = await api.query.authorMapping.nimbusLookup(CHARLETH_ADDRESS);
        expect(nimbusLookup.isSome).to.be.true;
        expect(nimbusLookup.unwrapOr(null)).to.not.equal(null);
        expect(nimbusLookup.unwrap().toString()).to.equal(newKeys[0]);
      },
    });
  },
});

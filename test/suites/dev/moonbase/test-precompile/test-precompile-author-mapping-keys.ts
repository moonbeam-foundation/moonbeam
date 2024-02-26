import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { FAITH_ADDRESS, FAITH_PRIVATE_KEY, getBlockExtrinsic } from "@moonwall/util";
import {
  concatOriginalKeys,
  originalKeys,
  setAuthorMappingKeysViaPrecompile,
} from "../../../../helpers/precompiles.js";

describeSuite({
  id: "D012910",
  title: "Precompile Author Mapping - Set Faith first time keys",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async function () {
      log(`Setting account ${FAITH_ADDRESS} keys: ${concatOriginalKeys}`);
      await setAuthorMappingKeysViaPrecompile(
        context,
        FAITH_ADDRESS,
        FAITH_PRIVATE_KEY,
        concatOriginalKeys
      );
    });

    it({
      id: "T01",
      title: "should succeed",
      test: async function () {
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
            .isSucceed
        ).to.be.true;
      },
    });

    it({
      id: "T02",
      title: "should send KeysRegistered event",
      test: async function () {
        const { events } = await getBlockExtrinsic(
          context.polkadotJs(),
          await context.polkadotJs().rpc.chain.getBlockHash(),
          "ethereum",
          "transact"
        );
        expect(events.find((e) => e.section == "authorMapping" && e.method == "KeysRegistered")).to
          .exist;
      },
    });

    it({
      id: "T03",
      title: "should set new keys",
      test: async function () {
        const mapping = (await context
          .polkadotJs()
          .query.authorMapping.mappingWithDeposit(originalKeys[0])) as any;
        expect(mapping.isSome).to.be.true;
        expect(mapping.unwrap().account.toString()).to.equal(FAITH_ADDRESS);
        expect(mapping.unwrap().keys_.toString()).to.equal(originalKeys[1]);
      },
    });

    it({
      id: "T04",
      title: "should set correct nimbus lookup",
      test: async function () {
        const nimbusLookup = await context
          .polkadotJs()
          .query.authorMapping.nimbusLookup(FAITH_ADDRESS);
        expect(nimbusLookup.isSome).to.be.true;
        expect(nimbusLookup.unwrap().toString()).to.equal(originalKeys[0]);
      },
    });
  },
});

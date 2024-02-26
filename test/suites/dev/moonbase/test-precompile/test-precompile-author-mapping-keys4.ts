import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import {
  FAITH_ADDRESS,
  FAITH_PRIVATE_KEY,
  PRECOMPILE_AUTHOR_MAPPING_ADDRESS,
  getBlockExtrinsic,
} from "@moonwall/util";
import {
  concatOriginalKeys,
  originalKeys,
  setAuthorMappingKeysViaPrecompile,
} from "../../../../helpers/precompiles.js";
import { sendPrecompileTx } from "../../../../helpers";

const SELECTORS = {
  set_keys: "bcb24ddc",
  remove_keys: "3b6c4284",
};

describeSuite({
  id: "D012913",
  title: "Precompile Author Mapping - Remove Faith keys",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async function () {
      await setAuthorMappingKeysViaPrecompile(
        context,
        FAITH_ADDRESS,
        FAITH_PRIVATE_KEY,
        concatOriginalKeys
      );
      // Remove the keys

      await sendPrecompileTx(
        context,
        PRECOMPILE_AUTHOR_MAPPING_ADDRESS,
        SELECTORS,
        FAITH_ADDRESS,
        FAITH_PRIVATE_KEY,
        "remove_keys",
        []
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
      title: "should send KeysRemoved event",
      test: async function () {
        const { events } = await getBlockExtrinsic(
          context.polkadotJs(),
          await context.polkadotJs().rpc.chain.getBlockHash(),
          "ethereum",
          "transact"
        );
        expect(events.find((e) => e.section == "authorMapping" && e.method == "KeysRemoved")).to
          .exist;
      },
    });

    it({
      id: "T03",
      title: "should remove keys",
      test: async function () {
        const mapping = await context
          .polkadotJs()
          .query.authorMapping.mappingWithDeposit(originalKeys[0]);
        expect(mapping.isNone).to.be.true;
      },
    });

    it({
      id: "T04",
      title: "should remove nimbus mapping",
      test: async function () {
        const nimbusLookup = await context
          .polkadotJs()
          .query.authorMapping.nimbusLookup(FAITH_ADDRESS);
        expect(nimbusLookup.isNone).to.be.true;
      },
    });
  },
});

import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import {
  ETHAN_ADDRESS,
  ETHAN_PRIVATE_KEY,
  FAITH_ADDRESS,
  FAITH_PRIVATE_KEY,
  PRECOMPILE_AUTHOR_MAPPING_ADDRESS,
  createViemTransaction,
  getBlockExtrinsic,
} from "@moonwall/util";
import { encodeFunctionData } from "viem";
import { originalKeys, setAuthorMappingKeysViaPrecompile } from "../../../../helpers";

describeSuite({
  id: "D010306",
  title: "Precompile Author Mapping - Update someone else nimbus key",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const concatOriginalKeys = `0x${originalKeys.map((key) => key.slice(2)).join("")}`;

    beforeAll(async function () {
      await setAuthorMappingKeysViaPrecompile(
        context,
        FAITH_ADDRESS,
        FAITH_PRIVATE_KEY,
        concatOriginalKeys
      );
    });

    it({
      id: "T01",
      title: "should revert",
      test: async function () {
        const { abi } = fetchCompiledContract("AuthorMapping");
        // Setting same key but with ethan
        await context.createBlock(
          createViemTransaction(context, {
            to: PRECOMPILE_AUTHOR_MAPPING_ADDRESS,
            privateKey: ETHAN_PRIVATE_KEY,
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
        // ethereum revert is still a successful substrate extrinsic
        expect(resultEvent?.method).to.equal("ExtrinsicSuccess");
        expect(
          (events.find((e) => e.section === "ethereum" && e.method === "Executed")?.data[3] as any)
            .isRevert
        ).to.be.true;
      },
    });

    it({
      id: "T02",
      title: "should not send any authorMapping event",
      test: async function () {
        const { events } = await getBlockExtrinsic(
          context.polkadotJs(),
          await context.polkadotJs().rpc.chain.getBlockHash(),
          "ethereum",
          "transact"
        );
        expect(events.find((e) => e.section === "authorMapping")).to.not.exist;
      },
    });

    it({
      id: "T03",
      title: "should keep the same keys to Faith",
      test: async function () {
        const mapping = await context
          .polkadotJs()
          .query.authorMapping.mappingWithDeposit(originalKeys[0]);
        expect(mapping.isSome).to.be.true;
        expect(mapping.unwrap().account.toString()).to.equal(FAITH_ADDRESS);
        expect(mapping.unwrap().keys_.toString()).to.equal(originalKeys[1]);
      },
    });

    it({
      id: "T04",
      title: "should not set nimbus lookup to Ethan",
      test: async function () {
        const nimbusLookup = await context
          .polkadotJs()
          .query.authorMapping.nimbusLookup(ETHAN_ADDRESS);
        expect(nimbusLookup.isNone).to.be.true;
      },
    });

    it({
      id: "T05",
      title: "should keep the same nimbus lookup to Faith",
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

// // Testing invalid inputs

// describeDevMoonbeam("Precompile Author Mapping - Set Faith only 1 key", (context) => {
//   it("should fail", async function () {
//     await setAuthorMappingKeysViaPrecompile(context, faith.address, FAITH_PRIVATE_KEY,
// originalKeys[0]);
//     const { extrinsic, events, resultEvent } = await getBlockExtrinsic(
//       context.polkadotJs(),
//       await context.polkadotJs().rpc.chain.getBlockHash(),
//       "ethereum",
//       "transact"
//     );

//     expect(extrinsic).to.exist;
//     expect(resultEvent.method).to.equal("ExtrinsicSuccess");
//     expect(
//       (events.find((e) => e.section === "ethereum" && e.method === "Executed").data[3] as any)
//         .isRevert
//     ).to.be.true;
//   });
// });

// describeDevMoonbeam("Precompile Author Mapping - Set Faith mapping with 0 keys", (context) => {
//   it("should fail", async function () {
//     await setAuthorMappingKeysViaPrecompile(context, faith.address, FAITH_PRIVATE_KEY, "0x");
//     const { extrinsic, events, resultEvent } = await getBlockExtrinsic(
//       context.polkadotJs(),
//       await context.polkadotJs().rpc.chain.getBlockHash(),
//       "ethereum",
//       "transact"
//     );

//     expect(extrinsic).to.exist;
//     expect(resultEvent.method).to.equal("ExtrinsicSuccess");
//     expect(
//       (events.find((e) => e.section === "ethereum" && e.method === "Executed").data[3] as any)
//         .isRevert
//     ).to.be.true;
//   });
// });

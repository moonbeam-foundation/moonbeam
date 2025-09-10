import "@moonbeam-network/api-augment";
import { ALITH_ADDRESS, BALTATHAR_SESSION_ADDRESS, CHARLETH_SESSION_ADDRESS } from "@moonwall/util";
import { getMappingInfo } from "../../../../helpers";
import { expect, describeSuite } from "@moonwall/cli";

describeSuite({
  id: "D020208",
  title: "Author Mapping - registered can rotate",
  foundationMethods: "dev",
  testCases: ({ context, log, it }) => {
    it({
      id: "T01",
      title: "should succeed in rotating account ids for an author",
      test: async function () {
        await context.createBlock(
          context.polkadotJs().tx.authorMapping.addAssociation(BALTATHAR_SESSION_ADDRESS)
        );
        expect((await getMappingInfo(context, BALTATHAR_SESSION_ADDRESS))?.account).to.eq(
          ALITH_ADDRESS
        );

        await context.createBlock(
          context
            .polkadotJs()
            .tx.authorMapping.updateAssociation(BALTATHAR_SESSION_ADDRESS, CHARLETH_SESSION_ADDRESS)
        );
        expect(await getMappingInfo(context, BALTATHAR_SESSION_ADDRESS)).toBeUndefined();
        expect((await getMappingInfo(context, CHARLETH_SESSION_ADDRESS))?.account).to.eq(
          ALITH_ADDRESS
        );

        await context.createBlock();
      },
    });
  },
});

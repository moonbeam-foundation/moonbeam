import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, baltathar, BALTATHAR_SESSION_ADDRESS } from "@moonwall/util";
import { getMappingInfo } from "../../../../helpers";

describeSuite({
  id: "D020205",
  title: "Author Mapping - non author clearing",
  foundationMethods: "dev",
  testCases: ({ context, log, it }) => {
    it({
      id: "T01",
      title: "should not succeed in clearing an association for a non-author",
      test: async function () {
        const api = context.polkadotJs();
        await context.createBlock(api.tx.authorMapping.addAssociation(BALTATHAR_SESSION_ADDRESS));
        expect((await getMappingInfo(context, BALTATHAR_SESSION_ADDRESS))!.account).to.eq(
          ALITH_ADDRESS
        );

        const { result } = await context.createBlock(
          api.tx.authorMapping.clearAssociation(BALTATHAR_SESSION_ADDRESS),
          { allowFailures: true, signer: baltathar }
        );

        expect(result?.events.length === 4);
        expect(api.events.system.ExtrinsicFailed.is(result?.events[4].event)).to.be.true;
      },
    });
  },
});

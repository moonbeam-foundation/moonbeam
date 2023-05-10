import "@moonbeam-network/api-augment";
import { alith, baltathar, BALTATHAR_SESSION_ADDRESS } from "@moonwall/util";
import { expect, describeSuite, beforeAll } from "@moonwall/cli";
import { getMappingInfo } from "../../../../helpers/common.js";

describeSuite({
  id: "D224",
  title: "Author Mapping - non author clearing",
  foundationMethods: "dev",
  testCases: ({ context, log, it }) => {
    it({
      id: "T01",
      title: "should not succeed in clearing an association for a non-author",
      test: async function () {
        const api = context.polkadotJs({ type: "moon" });
        await context.createBlock(api.tx.authorMapping.addAssociation(BALTATHAR_SESSION_ADDRESS));
        expect((await getMappingInfo(context, BALTATHAR_SESSION_ADDRESS)).account).to.eq(
          alith.address
        );

        const {
          result: { events },
        } = await context.createBlock(
          api.tx.authorMapping.clearAssociation(BALTATHAR_SESSION_ADDRESS).signAsync(baltathar),
          { allowFailures: true }
        );

        expect(events.length === 4);
        expect(api.events.treasury.Deposit.is(events[2].event)).to.be.true;
        expect(api.events.system.ExtrinsicFailed.is(events[4].event)).to.be.true;
      },
    });
  },
});

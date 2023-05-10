import "@moonbeam-network/api-augment";
import { BALTATHAR_SESSION_ADDRESS } from "@moonwall/util";
import { expect, describeSuite, beforeAll } from "@moonwall/cli";
import { getMappingInfo } from "../../../../helpers/common.js";

describeSuite({
  id: "D229",
  title: "Author Mapping - unregistered author cannot clear association",
  foundationMethods: "dev",
  testCases: ({ context, log, it }) => {
    it({
      id: "",
      title: "should not succeed in clearing an association for an unregistered author",
      test: async function () {
        expect(await getMappingInfo(context, BALTATHAR_SESSION_ADDRESS)).to.eq(null);
        const api = context.polkadotJs({ type: "moon" });
        const {
          result: { events },
        } = await context.createBlock(
          api.tx.authorMapping.clearAssociation(BALTATHAR_SESSION_ADDRESS),
          { allowFailures: true }
        );
        expect(events.length === 6);
        expect(api.events.system.NewAccount.is(events[2].event)).to.be.true;
        expect(api.events.balances.Endowed.is(events[3].event)).to.be.true;
        expect(api.events.treasury.Deposit.is(events[4].event)).to.be.true;
        expect(api.events.system.ExtrinsicFailed.is(events[6].event)).to.be.true;
      },
    });
  },
});

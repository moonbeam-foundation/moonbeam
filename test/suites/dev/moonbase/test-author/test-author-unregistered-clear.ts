import "@moonbeam-network/api-augment";
import { BALTATHAR_SESSION_ADDRESS } from "@moonwall/util";
import { expect, describeSuite } from "@moonwall/cli";
import { getMappingInfo } from "../../../../helpers";

describeSuite({
  id: "D020213",
  title: "Author Mapping - unregistered author cannot clear association",
  foundationMethods: "dev",
  testCases: ({ context, log, it }) => {
    it({
      id: "",
      title: "should not succeed in clearing an association for an unregistered author",
      test: async function () {
        expect(await getMappingInfo(context, BALTATHAR_SESSION_ADDRESS)).toBeUndefined();
        const api = context.polkadotJs();
        const { result } = await context.createBlock(
          api.tx.authorMapping.clearAssociation(BALTATHAR_SESSION_ADDRESS),
          { allowFailures: true }
        );
        expect(result?.events.length === 6);
        expect(api.events.system.NewAccount.is(result?.events[2].event)).to.be.true;
        expect(api.events.balances.Endowed.is(result?.events[3].event)).to.be.true;
        expect(api.events.system.ExtrinsicFailed.is(result?.events[6].event)).to.be.true;
      },
    });
  },
});

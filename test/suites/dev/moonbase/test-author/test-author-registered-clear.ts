import "@moonbeam-network/api-augment";
import { alith, ALITH_ADDRESS, BALTATHAR_SESSION_ADDRESS } from "@moonwall/util";
import { expect, describeSuite } from "@moonwall/cli";
import { getMappingInfo } from "../../../../helpers";

describeSuite({
  id: "D020207",
  title: "Author Mapping - registered author can clear (de register)",
  foundationMethods: "dev",
  testCases: ({ context, log, it }) => {
    it({
      id: "T01",
      title: "should succeed in clearing an association",
      test: async function () {
        const api = context.polkadotJs();
        await context.createBlock(
          api.tx.authorMapping.addAssociation(BALTATHAR_SESSION_ADDRESS).signAsync(alith)
        );
        expect((await getMappingInfo(context, BALTATHAR_SESSION_ADDRESS))!.account).to.eq(
          ALITH_ADDRESS
        );

        const { result } = await context.createBlock(
          api.tx.authorMapping.clearAssociation(BALTATHAR_SESSION_ADDRESS)
        );
        //check events
        expect(result?.events.length === 6);
        expect(api.events.balances.Unreserved.is(result?.events[1].event)).to.be.true;
        expect(api.events.authorMapping.KeysRemoved.is(result?.events[2].event)).to.be.true;
        expect(api.events.system.ExtrinsicSuccess.is(result?.events[6].event)).to.be.true;

        // check mapping
        expect(await getMappingInfo(context, BALTATHAR_SESSION_ADDRESS)).toBeUndefined();
      },
    });
  },
});

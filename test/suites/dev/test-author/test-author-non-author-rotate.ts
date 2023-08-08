import "@moonbeam-network/api-augment";
import {
  alith,
  baltathar,
  BALTATHAR_SESSION_ADDRESS,
  CHARLETH_SESSION_ADDRESS,
} from "@moonwall/util";
import { expect, describeSuite, beforeAll } from "@moonwall/cli";
import { ApiPromise } from "@polkadot/api";
import { getMappingInfo } from "../../../helpers/common.js";

describeSuite({
  id: "D0206",
  title: "Author Mapping - non-author cannot rotate",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let api: ApiPromise;

    beforeAll(async () => {
      api = context.polkadotJs();
    });

    it({
      id: "T01",
      title: "should fail rotating account ids if not an author",
      test: async function () {
        await context.createBlock(api.tx.authorMapping.addAssociation(BALTATHAR_SESSION_ADDRESS));
        expect((await getMappingInfo(context, BALTATHAR_SESSION_ADDRESS))!.account).to.eq(
          alith.address
        );

        await context.createBlock(
          api.tx.authorMapping
            .updateAssociation(BALTATHAR_SESSION_ADDRESS, CHARLETH_SESSION_ADDRESS)
            .signAsync(baltathar),
          { allowFailures: true }
        );
        expect((await getMappingInfo(context, BALTATHAR_SESSION_ADDRESS))!.account).to.eq(
          alith.address
        );
        expect(await getMappingInfo(context, CHARLETH_SESSION_ADDRESS)).to.eq(null);

        await context.createBlock();
      },
    });
  },
});

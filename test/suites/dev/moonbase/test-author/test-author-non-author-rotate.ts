import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  baltathar,
  BALTATHAR_SESSION_ADDRESS,
  CHARLETH_SESSION_ADDRESS,
} from "@moonwall/util";
import type { ApiPromise } from "@polkadot/api";
import { getMappingInfo } from "../../../../helpers";

describeSuite({
  id: "D020206",
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
          ALITH_ADDRESS
        );

        await context.createBlock(
          api.tx.authorMapping.updateAssociation(
            BALTATHAR_SESSION_ADDRESS,
            CHARLETH_SESSION_ADDRESS
          ),
          { allowFailures: true, signer: baltathar }
        );
        expect((await getMappingInfo(context, BALTATHAR_SESSION_ADDRESS))!.account).to.eq(
          ALITH_ADDRESS
        );
        expect(await getMappingInfo(context, CHARLETH_SESSION_ADDRESS)).toBeUndefined();

        await context.createBlock();
      },
    });
  },
});

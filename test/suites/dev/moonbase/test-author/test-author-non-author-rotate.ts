import "@moonbeam-network/api-augment";
import {
  ALITH_ADDRESS,
  BALTATHAR_SESSION_ADDRESS,
  CHARLETH_SESSION_ADDRESS,
  baltathar,
  beforeAll,
  describeSuite,
  expect,
} from "moonwall";
import type { ApiPromise } from "@polkadot/api";
import { getMappingInfo } from "../../../../helpers";

describeSuite({
  id: "D020206",
  title: "Author Mapping - non-author cannot rotate",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
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

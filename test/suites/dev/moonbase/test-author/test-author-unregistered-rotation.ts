import "@moonbeam-network/api-augment";
import {
  BALTATHAR_SESSION_ADDRESS,
  CHARLETH_SESSION_ADDRESS,
  alith,
  describeSuite,
  expect,
} from "moonwall";
import { getMappingInfo } from "../../../../helpers";

describeSuite({
  id: "D020214",
  title: "Author Mapping - unregistered cannot rotate",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    it({
      id: "T01",
      title: "should fail rotating account ids if not registered",
      test: async function () {
        await context.createBlock(
          context
            .polkadotJs()
            .tx.authorMapping.updateAssociation(
              BALTATHAR_SESSION_ADDRESS,
              CHARLETH_SESSION_ADDRESS
            ),
          { allowFailures: true, signer: alith }
        );
        expect(await getMappingInfo(context, BALTATHAR_SESSION_ADDRESS)).toBeUndefined();
        expect(await getMappingInfo(context, CHARLETH_SESSION_ADDRESS)).toBeUndefined();

        await context.createBlock();
      },
    });
  },
});

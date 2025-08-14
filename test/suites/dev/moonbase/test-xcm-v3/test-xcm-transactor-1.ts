import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { alith } from "@moonwall/util";

describeSuite({
  id: "D024036",
  title: "Precompiles - xcm transactor",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      // register index 0 for Alith
      await context.createBlock(
        context
          .polkadotJs()
          .tx.sudo.sudo(context.polkadotJs().tx.xcmTransactor.register(alith.address, 0))
      );
    });

    it({
      id: "T01",
      title: "allows to retrieve index through precompiles",
      test: async function () {
        const resp = await context.polkadotJs().query.xcmTransactor.indexToAccount(0);
        expect(resp.toString()).to.eq(alith.address);
      },
    });
  },
});

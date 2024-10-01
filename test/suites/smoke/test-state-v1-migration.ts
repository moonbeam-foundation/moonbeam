import "@moonbeam-network/api-augment/moonbase";
import { ApiPromise } from "@polkadot/api";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";

describeSuite({
  id: "S27",
  title: `State V1 Migration status should not be in an error state`,
  foundationMethods: "read_only",
  testCases: ({ context, it, log }) => {
    let paraApi: ApiPromise;

    beforeAll(async function () {
      paraApi = context.polkadotJs("para");
    });

    it({
      id: "C100",
      title: "Migration status should not be in an error state",
      test: async function (context) {
        if (paraApi.consts.system.version.specVersion.toNumber() < 3300) {
          context.skip();
        }
        const stateMigrationStatus =
          await paraApi.query.moonbeamLazyMigrations.stateMigrationStatusValue();
        const isError = stateMigrationStatus.toString().toLowerCase().includes("error");
        expect(isError).to.be.false;
      },
    });
  },
});

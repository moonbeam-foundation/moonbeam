import "@moonbeam-network/api-augment/moonbase";
import { ApiPromise } from "@polkadot/api";
import { beforeAll, describeSuite, expect, MoonwallContext, ProviderMap } from "@moonwall/cli";

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
        const stateMigrationStatus =
          await paraApi.query.moonbeamLazyMigrations.stateMigrationStatusValue();
        const isError = stateMigrationStatus.toJSON()?.toString().toLowerCase().includes("error");
        expect(isError).to.be.true;
      },
    });
  },
});

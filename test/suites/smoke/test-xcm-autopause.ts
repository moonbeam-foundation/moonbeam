import "@moonbeam-network/api-augment/moonbase";
import { ApiPromise } from "@polkadot/api";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";

describeSuite({
  id: "S26",
  title: `XCM Mode should be equal to Normal`,
  foundationMethods: "read_only",
  testCases: ({ context, it, log }) => {
    let paraApi: ApiPromise;

    beforeAll(async function () {
      paraApi = context.polkadotJs("para");
    });

    it({
      id: "C100",
      title: "XCM Mode should be equal to Normal",
      test: async function (context) {
        if ((await paraApi.consts.system.version.specVersion).toNumber() < 3200) {
            context.skip();
        }

        // XCM Mode should be equal to Normal
        expect((await paraApi.query.emergencyParaXcm.mode()).isNormal).to.be.true;
      },
    });
  },
});

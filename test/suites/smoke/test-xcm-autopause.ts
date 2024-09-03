import "@moonbeam-network/api-augment/moonbase";
import { describeSuite, expect } from "@moonwall/cli";

describeSuite({
  id: "S26",
  title: `XCM Mode should be equal to Normal`,
  foundationMethods: "read_only",
  testCases: ({ context, it, log }) => {
    it({
      id: "C100",
      title: "XCM Mode should be equal to Normal",
      test: async function (context) {
        if ((await context.polkadotJs().consts.system.version.specVersion) < 3200) {
          return context.skip();
        }

        // XCM Mode should be equal to Normal
        expect((await context.polkadotJs().query.emergencyParaXcm.mode()).isNormal).to.be.true;
      },
    });
  },
});

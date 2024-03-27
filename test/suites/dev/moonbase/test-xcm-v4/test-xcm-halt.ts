import "@moonbeam-network/api-augment";
import { describeSuite, beforeEach, expect } from "@moonwall/cli";
import { ApiPromise } from "@polkadot/api";

describeSuite({
  id: "D014087",
  title: "XCM halt",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let polkadotJs: ApiPromise;

    beforeEach(async function () {
      polkadotJs = context.polkadotJs();
    });

    it({
      id: "T01",
      title: "Testing",
      test: async function () {
        const xcmPaused = await polkadotJs.query.emergencyParaXcm.xcmPaused();
        console.log(xcmPaused.toHuman());

      },
    });
  },
});

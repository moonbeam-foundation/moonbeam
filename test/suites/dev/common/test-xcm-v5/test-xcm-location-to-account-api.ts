import { describeSuite, expect } from "@moonwall/cli";
import { RELAY_V3_SOURCE_LOCATION } from "../../../../helpers/assets";

describeSuite({
  id: "D010713",
  title: "XCM - LocationToAccountApi",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    it({
      id: "T01",
      title: "Should succeed calling LocationToAccountApi::convertLocation",
      test: async function () {
        const convertLocation = await context
          .polkadotJs()
          .call.locationToAccountApi.convertLocation(RELAY_V3_SOURCE_LOCATION);

        expect(convertLocation.isOk).to.be.true;
        expect(convertLocation.asOk.toHuman()).to.eq("0x506172656E740000000000000000000000000000");
      },
    });
  },
});

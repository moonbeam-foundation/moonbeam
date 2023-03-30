import "@moonbeam-network/api-augment";
import { describeSuite, beforeAll, expect } from "@moonsong-labs/moonwall-cli";
import { ApiDecoration } from "@polkadot/api/types";

describeSuite({
  id: "S100",
  title: `Verify author filter consistency`,
  foundationMethods: "read_only",
  testCases: ({ context, it, log }) => {
    let atBlockNumber: number = 0;
    let apiAt: ApiDecoration<"promise"> = null;
    let specVersion: number = 0;

    beforeAll(async function () {
      atBlockNumber = (await context.getSubstrateApi().rpc.chain.getHeader()).number.toNumber();
      apiAt = await context
        .getSubstrateApi()
        .at(await context.getSubstrateApi().rpc.chain.getBlockHash(atBlockNumber));
      specVersion = (await apiAt.query.system.lastRuntimeUpgrade()).unwrap().specVersion.toNumber();
    });

    it({
      id: "C100",
      title: `should have eligibility > 0`,
      test: async function () {
        if (specVersion < 1500) {
          const eligibilityRatio = await apiAt.query.authorFilter.eligibleRatio();
          expect(eligibilityRatio.toBigInt() > 0n).to.be.true;
        }

        if (specVersion >= 1500) {
          const eligibilityCount = await apiAt.query.authorFilter.eligibleCount();
          expect(eligibilityCount.toNumber() > 0).to.be.true;
        }

        log(`Verified eligibility`);
      },
    });
  },
});

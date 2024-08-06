import "@moonbeam-network/api-augment";
import { describeSuite, beforeAll, expect } from "@moonwall/cli";
import { ApiDecoration } from "@polkadot/api/types";
import { ApiPromise } from "@polkadot/api";

describeSuite({
  id: "S02",
  title: `Verify author filter consistency`,
  foundationMethods: "read_only",
  testCases: ({ context, it, log }) => {
    let atBlockNumber: number = 0;
    let apiAt: ApiDecoration<"promise">;
    let specVersion: number = 0;
    let paraApi: ApiPromise;

    beforeAll(async function () {
      paraApi = context.polkadotJs("para");
      atBlockNumber = (await paraApi.rpc.chain.getHeader()).number.toNumber();
      apiAt = await paraApi.at(await paraApi.rpc.chain.getBlockHash(atBlockNumber));
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

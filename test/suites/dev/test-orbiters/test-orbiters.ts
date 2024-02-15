import { describeSuite, expect, beforeAll } from "@moonwall/cli";
import "@moonbeam-network/api-augment";
import { CHARLETH_ADDRESS, BALTATHAR_ADDRESS, alith, setupLogger } from "@moonwall/util";
import { parseEther, formatEther, Signer } from "ethers";
import { ApiPromise } from "@polkadot/api";

describeSuite({
  id: "TX01",
  title: "Orbiters",
  foundationMethods: "dev",
  testCases: ({ it, context, log }) => {
    it({
      id: "T01",
      title: "Marking orbiters offline is a noop",
      test: async function () {
        const psQuery = context.polkadotJs().query.parachainStaking;
        const collators = psQuery.selectedCandidates();
        const candidates = psQuery.selectedPool();

        console.log(collators);
        console.log(candidates);
      },
    });
  }
});

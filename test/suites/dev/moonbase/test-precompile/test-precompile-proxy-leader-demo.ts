import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { BALTATHAR_ADDRESS, CHARLETH_ADDRESS, DOROTHY_ADDRESS, GLMR } from "@moonwall/util";
import { setupPoolWithParticipants } from "../../../../helpers";

describeSuite({
  id: "D012962",
  title: "Proxy Leader Demo - Preparing Participation Pool",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let leaderContractAddress: `0x${string}`;

    beforeAll(async function () {
      leaderContractAddress = await setupPoolWithParticipants(context);
    });

    it({
      id: "T01",
      title: "should have a pool of 3 tokens",
      test: async function () {
        expect(
          await context.readContract!({
            contractAddress: leaderContractAddress,
            contractName: "ProxyLeaderDemo",
            functionName: "pooledAmount",
          })
        ).to.equal(3n * GLMR);
      },
    });

    it({
      id: "T02",
      title: "should have a balance of 3 tokens",
      test: async function () {
        const freeBalance = (
          await context.polkadotJs().query.system.account(leaderContractAddress)
        ).data.free.toBigInt();

        expect(freeBalance).to.equal(3n * GLMR);
      },
    });

    it({
      id: "T03",
      title: "should have 3 participants",
      test: async function () {
        expect(
          await context.readContract!({
            contractAddress: leaderContractAddress,
            contractName: "ProxyLeaderDemo",
            functionName: "getParticipants",
          })
        ).to.deep.equal([BALTATHAR_ADDRESS, CHARLETH_ADDRESS, DOROTHY_ADDRESS]);
      },
    });
  },
});

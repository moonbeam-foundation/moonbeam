import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { expectEVMResult, setupPoolWithParticipants } from "../../../../helpers";

describeSuite({
  id: "D022850",
  title: "Proxy Leader Demo - Start Voting",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let leaderContractAddress: `0x${string}`;

    beforeAll(async function () {
      leaderContractAddress = await setupPoolWithParticipants(context);
    });

    it({
      id: "T01",
      title: "should be able to start",
      test: async function () {
        expect(
          await context.readContract!({
            contractName: "ProxyLeaderDemo",
            contractAddress: leaderContractAddress,
            functionName: "isVoting",
          })
        ).to.be.false;

        const rawTx = context.writeContract!({
          contractName: "ProxyLeaderDemo",
          contractAddress: leaderContractAddress,
          functionName: "startVoting",
          rawTxOnly: true,
        });
        const { result } = await context.createBlock(rawTx);

        expectEVMResult(result!.events, "Succeed");

        expect(
          await context.readContract!({
            contractName: "ProxyLeaderDemo",
            contractAddress: leaderContractAddress,
            functionName: "isVoting",
          })
        ).to.be.true;
      },
    });
  },
});

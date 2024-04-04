import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  BALTATHAR_ADDRESS,
  BALTATHAR_PRIVATE_KEY,
  CHARLETH_ADDRESS,
  DOROTHY_ADDRESS,
} from "@moonwall/util";
import { setupPoolWithParticipants, expectEVMResult } from "../../../../helpers";

describeSuite({
  id: "D012862",
  title: "Proxy Leader Demo - Vote",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let leaderContractAddress: `0x${string}`;

    beforeAll(async function () {
      leaderContractAddress = await setupPoolWithParticipants(context);

      const rawTx = context.writeContract!({
        contractName: "ProxyLeaderDemo",
        contractAddress: leaderContractAddress,
        functionName: "startVoting",
        rawTxOnly: true,
      });
      const { result } = await context.createBlock(rawTx);
      expectEVMResult(result!.events, "Succeed");
    });

    it({
      id: "T01",
      title: "should not be able to vote if non-participant",
      test: async function () {
        expect(
          await context.readContract!({
            contractAddress: leaderContractAddress,
            contractName: "ProxyLeaderDemo",
            functionName: "canVote",
            args: [ALITH_ADDRESS],
          })
        ).to.be.false;

        const rawTx = context.writeContract!({
          contractName: "ProxyLeaderDemo",
          contractAddress: leaderContractAddress,
          functionName: "vote",
          args: [CHARLETH_ADDRESS, DOROTHY_ADDRESS],
          rawTxOnly: true,
          gas: 1000000n,
        });
        const { result } = await context.createBlock(rawTx);
        expectEVMResult(result!.events, "Revert");
      },
    });

    it({
      id: "T02",
      title: "should not be able to vote for non-participant",
      test: async function () {
        expect(
          await context.readContract!({
            contractAddress: leaderContractAddress,
            contractName: "ProxyLeaderDemo",
            functionName: "canVote",
            args: [BALTATHAR_ADDRESS],
          })
        ).to.be.true;

        const rawTx = context.writeContract!({
          contractName: "ProxyLeaderDemo",
          contractAddress: leaderContractAddress,
          functionName: "vote",
          args: [ALITH_ADDRESS, DOROTHY_ADDRESS],
          rawTxOnly: true,
          privateKey: BALTATHAR_PRIVATE_KEY,
          gas: 1000000n,
        });
        const { result } = await context.createBlock(rawTx);
        expectEVMResult(result!.events, "Revert");
      },
    });

    it({
      id: "T03",
      title: "should be able to vote for participant when participant",
      test: async function () {
        expect(
          await context.readContract!({
            contractAddress: leaderContractAddress,
            contractName: "ProxyLeaderDemo",
            functionName: "canVote",
            args: [BALTATHAR_ADDRESS],
          })
        ).to.be.true;

        const rawTx = context.writeContract!({
          contractName: "ProxyLeaderDemo",
          contractAddress: leaderContractAddress,
          functionName: "vote",
          args: [CHARLETH_ADDRESS, DOROTHY_ADDRESS],
          rawTxOnly: true,
          privateKey: BALTATHAR_PRIVATE_KEY,
          gas: 1_000_000n,
        });
        const { result } = await context.createBlock(rawTx);

        expectEVMResult(result!.events, "Succeed");
        expect(
          await context.readContract!({
            contractAddress: leaderContractAddress,
            contractName: "ProxyLeaderDemo",
            functionName: "canVote",
            args: [BALTATHAR_ADDRESS],
          })
        ).to.be.false;
      },
    });
  },
});

import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  BALTATHAR_ADDRESS,
  BALTATHAR_PRIVATE_KEY,
  CHARLETH_ADDRESS,
  DOROTHY_ADDRESS,
} from "@moonwall/util";
import { setupPoolWithParticipants } from "../../../helpers/precompiles.js";
import { expectEVMResult } from "../../../helpers/eth-transactions.js";

describeSuite({
  id: "D2544",
  title: "Proxy Leader Demo - Vote",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let leaderContractAddress: `0x${string}`;

    beforeAll(async function () {
      console.log("beforeAll Proxy Leader Demo");
      leaderContractAddress = await setupPoolWithParticipants(context);
      console.log("beforeAll Proxy Leader Demo 2");

      const rawTx = context.writeContract!({
        contractName: "ProxyLeaderDemo",
        contractAddress: leaderContractAddress,
        functionName: "startVoting",
        rawTxOnly: true,
      });
      console.log("beforeAll Proxy Leader Demo 3");
      const { result } = await context.createBlock(rawTx);
      console.log("beforeAll Proxy Leader Demo 4");
      expectEVMResult(result!.events, "Succeed");
      console.log("beforeAll Proxy Leader Demo 5");
    });

    it({
      id: "T01",
      title: "should not be able to vote if non-participant",
      test: async function () {
        console.log("beforeAll Proxy Leader Demo T01 1");
        expect(
          await context.readContract!({
            contractAddress: leaderContractAddress,
            contractName: "ProxyLeaderDemo",
            functionName: "canVote",
            args: [ALITH_ADDRESS],
          })
        ).to.be.false;
        console.log("beforeAll Proxy Leader Demo T01 2");

        const rawTx = context.writeContract!({
          contractName: "ProxyLeaderDemo",
          contractAddress: leaderContractAddress,
          functionName: "vote",
          args: [CHARLETH_ADDRESS, DOROTHY_ADDRESS],
          rawTxOnly: true,
          gas: 1000000n,
        });
        console.log("beforeAll Proxy Leader Demo T01 3");
        const { result } = await context.createBlock(rawTx);
        console.log("beforeAll Proxy Leader Demo T01 4");
        expectEVMResult(result!.events, "Revert");
        console.log("beforeAll Proxy Leader Demo T01 5");
      },
    });

    it({
      id: "T02",
      title: "should not be able to vote for non-participant",
      test: async function () {
        console.log("beforeAll Proxy Leader Demo T02 1");
        expect(
          await context.readContract!({
            contractAddress: leaderContractAddress,
            contractName: "ProxyLeaderDemo",
            functionName: "canVote",
            args: [BALTATHAR_ADDRESS],
          })
        ).to.be.true;
        console.log("beforeAll Proxy Leader Demo T02 2");

        const rawTx = context.writeContract!({
          contractName: "ProxyLeaderDemo",
          contractAddress: leaderContractAddress,
          functionName: "vote",
          args: [ALITH_ADDRESS, DOROTHY_ADDRESS],
          rawTxOnly: true,
          privateKey: BALTATHAR_PRIVATE_KEY,
          gas: 1000000n,
        });
        console.log("beforeAll Proxy Leader Demo T02 3");
        const { result } = await context.createBlock(rawTx);
        console.log("beforeAll Proxy Leader Demo T02 4");
        expectEVMResult(result!.events, "Revert");
        console.log("beforeAll Proxy Leader Demo T02 5");
      },
    });

    it({
      id: "T03",
      title: "should be able to vote for participant when participant",
      test: async function () {
        console.log("beforeAll Proxy Leader Demo T03 1");
        expect(
          await context.readContract!({
            contractAddress: leaderContractAddress,
            contractName: "ProxyLeaderDemo",
            functionName: "canVote",
            args: [BALTATHAR_ADDRESS],
          })
        ).to.be.true;
        console.log("beforeAll Proxy Leader Demo T03 2");

        const rawTx = context.writeContract!({
          contractName: "ProxyLeaderDemo",
          contractAddress: leaderContractAddress,
          functionName: "vote",
          args: [CHARLETH_ADDRESS, DOROTHY_ADDRESS],
          rawTxOnly: true,
          privateKey: BALTATHAR_PRIVATE_KEY,
          gas: 1_000_000n,
        });
        console.log("beforeAll Proxy Leader Demo T03 3");
        const { result } = await context.createBlock(rawTx);
        console.log("beforeAll Proxy Leader Demo T03 4");

        expectEVMResult(result!.events, "Succeed");
        console.log("beforeAll Proxy Leader Demo T03 5");
        expect(
          await context.readContract!({
            contractAddress: leaderContractAddress,
            contractName: "ProxyLeaderDemo",
            functionName: "canVote",
            args: [BALTATHAR_ADDRESS],
          })
        ).to.be.false;
        console.log("beforeAll Proxy Leader Demo T03 6");
      },
    });
  },
});

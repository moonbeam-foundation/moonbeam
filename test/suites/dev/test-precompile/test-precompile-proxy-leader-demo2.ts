import "@moonbeam-network/api-augment";
import { DevModeContext, beforeAll, describeSuite, expect } from "@moonwall/cli";
import {
  BALTATHAR_PRIVATE_KEY,
  CHARLETH_PRIVATE_KEY,
  DOROTHY_PRIVATE_KEY,
  createViemTransaction,
} from "@moonwall/util";
import { encodeFunctionData, parseEther } from "viem";
import { expectEVMResult } from "../../../helpers/eth-transactions.js";

const setupPoolWithParticipants = async (context: DevModeContext) => {
  const { contractAddress, abi } = await context.deployContract!("ProxyLeaderDemo");
  expect(contractAddress.length).toBeGreaterThan(3);

  // Adds participants
  for (const [privateKey] of [
    [BALTATHAR_PRIVATE_KEY],
    [CHARLETH_PRIVATE_KEY],
    [DOROTHY_PRIVATE_KEY],
  ]) {
    const rawTxn = createViemTransaction(context, {
      to: contractAddress,
      value: parseEther("1"),
      data: encodeFunctionData({
        abi,
        functionName: "joinPool",
      }),
      privateKey,
    });
    const { result } = await context.createBlock(rawTxn);
    expectEVMResult(result!.events, "Succeed");
  }
  return contractAddress;
};

describeSuite({
  id: "D2543",
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

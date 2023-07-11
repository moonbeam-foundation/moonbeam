import "@moonbeam-network/api-augment";
import { DevModeContext, beforeAll, describeSuite, expect } from "@moonwall/cli";
import {
  BALTATHAR_ADDRESS,
  BALTATHAR_PRIVATE_KEY,
  CHARLETH_ADDRESS,
  CHARLETH_PRIVATE_KEY,
  DOROTHY_ADDRESS,
  DOROTHY_PRIVATE_KEY,
  GLMR,
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
  id: "D2542",
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

import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import {
  BALTATHAR_ADDRESS,
  CHARLETH_ADDRESS,
  CHARLETH_PRIVATE_KEY,
  ETHAN_ADDRESS,
  ETHAN_PRIVATE_KEY,
  GLMR,
  MIN_GLMR_STAKING,
  ethan,
} from "@moonwall/util";
import { nToHex } from "@polkadot/util";
import { setupWithParticipants } from "../../../../helpers";

describeSuite({
  id: "D012967",
  title: "Proxy Call Staking Demo - Leave Participant",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let demoContractAddress: `0x${string}`;

    beforeAll(async function () {
      demoContractAddress = await setupWithParticipants(context);

      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, 1)
          .signAsync(ethan)
      );

      await context.writeContract!({
        contractAddress: demoContractAddress,
        contractName: "ProxyCallStakingDemo",
        functionName: "registerCandidate",
        args: [0],
        privateKey: ETHAN_PRIVATE_KEY,
      });
      await context.createBlock();

      await context.writeContract!({
        contractAddress: demoContractAddress,
        contractName: "ProxyCallStakingDemo",
        functionName: "leave",
        privateKey: CHARLETH_PRIVATE_KEY,
      });
      await context.createBlock();
    });

    it({
      id: "T01",
      title: "should have 1 participant",
      test: async function () {
        expect(
          await context.readContract!({
            contractAddress: demoContractAddress,
            contractName: "ProxyCallStakingDemo",
            functionName: "isParticipant",
            args: [BALTATHAR_ADDRESS],
          })
        ).to.be.true;

        expect(
          await context.readContract!({
            contractAddress: demoContractAddress,
            contractName: "ProxyCallStakingDemo",
            functionName: "isParticipant",
            args: [CHARLETH_ADDRESS],
          })
        ).to.be.false;
      },
    });

    it({
      id: "T02",
      title: "should have 1 candidate",
      test: async function () {
        expect(
          await context.readContract!({
            contractAddress: demoContractAddress,
            contractName: "ProxyCallStakingDemo",
            functionName: "isCandidate",
            args: [ETHAN_ADDRESS],
          })
        ).to.be.true;
      },
    });

    it({
      id: "T03",
      title: "should have scheduled leave from charleth to ethan",
      test: async function () {
        const delegationRequests = await context
          .polkadotJs()
          .query.parachainStaking.delegationScheduledRequests(ETHAN_ADDRESS);

        const currentRound = (
          await context.polkadotJs().query.parachainStaking.round()
        ).current.toNumber();

        const roundDelay = context
          .polkadotJs()
          .consts.parachainStaking.revokeDelegationDelay.toNumber();

        expect(delegationRequests.toJSON()).to.deep.equal([
          {
            delegator: CHARLETH_ADDRESS,
            whenExecutable: currentRound + roundDelay,
            action: {
              revoke: nToHex(1n * GLMR, { bitLength: 128 }),
            },
          },
        ]);
      },
    });
  },
});

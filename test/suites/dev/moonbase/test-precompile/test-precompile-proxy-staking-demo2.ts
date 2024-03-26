import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import {
  BALTATHAR_ADDRESS,
  CHARLETH_ADDRESS,
  DOROTHY_ADDRESS,
  DOROTHY_PRIVATE_KEY,
  ETHAN_ADDRESS,
  ETHAN_PRIVATE_KEY,
  GLMR,
  MIN_GLMR_STAKING,
  dorothy,
  ethan,
} from "@moonwall/util";
import { nToHex } from "@polkadot/util";
import { setupWithParticipants } from "../../../../helpers";

describeSuite({
  id: "D012966",
  title: "Proxy Call Staking Demo - New Participant",
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

      await context.createBlock(
        context.polkadotJs().tx.proxy.addProxy(demoContractAddress, "Staking", 0).signAsync(dorothy)
      );

      await context.writeContract!({
        contractAddress: demoContractAddress,
        contractName: "ProxyCallStakingDemo",
        functionName: "join",
        args: [0],
        privateKey: DOROTHY_PRIVATE_KEY,
      });
      await context.createBlock();
    });

    it({
      id: "T01",
      title: "should have 3 participants",
      test: async function () {
        expect(
          await context.readContract!({
            contractAddress: demoContractAddress,
            contractName: "ProxyCallStakingDemo",
            functionName: "isParticipant",
            args: [CHARLETH_ADDRESS],
          })
        ).to.be.true;
        expect(
          await context.readContract!({
            contractAddress: demoContractAddress,
            contractName: "ProxyCallStakingDemo",
            functionName: "isParticipant",
            args: [DOROTHY_ADDRESS],
          })
        ).to.be.true;
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
      title: "should have delegated all participants including dorothy to ethan",
      test: async function () {
        const delegations = await context
          .polkadotJs()
          .query.parachainStaking.topDelegations(ETHAN_ADDRESS);
        expect(delegations.toJSON()).to.deep.equal({
          delegations: [
            {
              owner: BALTATHAR_ADDRESS,
              amount: nToHex(1n * GLMR, { bitLength: 128 }),
            },
            {
              owner: CHARLETH_ADDRESS,
              amount: nToHex(1n * GLMR, { bitLength: 128 }),
            },
            {
              owner: DOROTHY_ADDRESS,
              amount: nToHex(1n * GLMR, { bitLength: 128 }),
            },
          ],
          total: nToHex(3n * GLMR, { bitLength: 128 }),
        });
      },
    });
  },
});

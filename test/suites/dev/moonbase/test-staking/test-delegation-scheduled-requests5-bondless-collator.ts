import "@moonbeam-network/api-augment";
import { describeSuite, beforeAll, expect } from "@moonwall/cli";
import { MIN_GLMR_DELEGATOR, alith, baltathar, ethan } from "@moonwall/util";
import { jumpToRound } from "../../../../helpers/block.js";

describeSuite({
  id: "D013430",
  title:
    "Staking - Delegation Scheduled Requests with bondless collator \
        - execute revoke after round delay",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let psTx: any;
    let psQuery: any;
    let sudo: any;
    let createBlock: any;

    beforeAll(async () => {
      psTx = context.polkadotJs().tx.parachainStaking;
      psQuery = context.polkadotJs().query.parachainStaking;
      sudo = context.polkadotJs().tx.sudo.sudo;
      createBlock = context.createBlock;

      await createBlock(sudo(psTx.forceJoinCandidates(baltathar.address, 0, 1)).signAsync(alith));
      await createBlock([
        sudo(psTx.setBlocksPerRound(10)).signAsync(alith),
        psTx.delegate(alith.address, MIN_GLMR_DELEGATOR, 0, 0).signAsync(ethan),
      ]);
      await createBlock(
        psTx.delegate(baltathar.address, MIN_GLMR_DELEGATOR, 0, 1).signAsync(ethan)
      );
      await createBlock(psTx.scheduleRevokeDelegation(alith.address).signAsync(ethan));

      // jump to exact executable Round
      const delegationRequests = await psQuery.delegationScheduledRequests(alith.address);
      await jumpToRound(context, delegationRequests[0].whenExecutable.toNumber() + 5);
    });

    it({
      id: "T01",
      title: "should succeed",
      test: async () => {
        await createBlock(
          psTx.executeDelegationRequest(ethan.address, alith.address).signAsync(ethan)
        );
        const delegatorState = await psQuery.delegatorState(ethan.address);
        const delegationRequestsAfter = await psQuery.delegationScheduledRequests(alith.address);
        expect(delegatorState.unwrap().delegations[0].amount.toBigInt()).toBe(MIN_GLMR_DELEGATOR);
        expect(delegatorState.unwrap().delegations[0].owner.toString()).toBe(baltathar.address);
        expect(delegationRequestsAfter.toJSON()).to.be.empty;
      },
    });
  },
});

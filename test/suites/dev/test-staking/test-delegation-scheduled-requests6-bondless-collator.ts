import "@moonbeam-network/api-augment";
import { describeSuite, beforeAll, expect } from "@moonwall/cli";
import { MIN_GLMR_DELEGATOR, alith, baltathar, ethan } from "@moonwall/util";
import { jumpToRound } from "../../../helpers/block.js";

describeSuite({
  id: "D2995",
  title:
    "Staking - Delegation Scheduled Requests with bondless collator \
    - execute revoke on last delegation",
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
        psTx.delegate(baltathar.address, MIN_GLMR_DELEGATOR, 0, 0).signAsync(ethan),
      ]);

      await createBlock(psTx.scheduleRevokeDelegation(baltathar.address).signAsync(ethan));

      const delegationRequests = await psQuery.delegationScheduledRequests(baltathar.address);
      await jumpToRound(context, delegationRequests[0].whenExecutable.toNumber());
    });

    it({
      id: "T01",
      title: "should succeed and leave as delegator",
      test: async () => {
        await createBlock(
          psTx.executeDelegationRequest(ethan.address, baltathar.address).signAsync(ethan)
        );
        const delegatorState = await psQuery.delegatorState(ethan.address);
        const delegationRequestsAfter = await psQuery.delegationScheduledRequests(
          baltathar.address
        );
        expect(delegatorState.isNone).to.be.true; // last delegation revoked, so delegator left
        expect(delegationRequestsAfter.isEmpty).toBe(true);
      },
    });
  },
});

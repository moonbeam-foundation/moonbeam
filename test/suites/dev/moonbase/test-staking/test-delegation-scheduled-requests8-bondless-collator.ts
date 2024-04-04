import "@moonbeam-network/api-augment";
import { describeSuite, beforeAll, expect } from "@moonwall/cli";
import { MIN_GLMR_DELEGATOR, alith, baltathar, ethan } from "@moonwall/util";

describeSuite({
  id: "D013436",
  title:
    "Staking - Delegation Scheduled Requests with bondless collator - cancel scheduled bond less",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const LESS_AMOUNT = 10n;
    let psTx: any;
    let psQuery: any;
    let psConst: any;
    let sudo: any;
    let createBlock: any;

    beforeAll(async () => {
      psTx = context.polkadotJs().tx.parachainStaking;
      psQuery = context.polkadotJs().query.parachainStaking;
      psConst = context.polkadotJs().consts.parachainStaking;
      sudo = context.polkadotJs().tx.sudo.sudo;
      createBlock = context.createBlock;

      await createBlock(sudo(psTx.forceJoinCandidates(baltathar.address, 0, 1)).signAsync(alith));
      await createBlock(
        [
          sudo(psTx.setBlocksPerRound(10)).signAsync(alith),
          psTx.delegate(baltathar.address, MIN_GLMR_DELEGATOR + LESS_AMOUNT, 0, 0).signAsync(ethan),
        ],
        { allowFailures: false }
      );

      await createBlock(
        psTx.scheduleDelegatorBondLess(baltathar.address, LESS_AMOUNT).signAsync(ethan)
      );
    });

    it({
      id: "T01",
      title: "should succeed",
      test: async () => {
        const currentRound = (await psQuery.round()).current.toNumber();
        const delegationRequests = await psQuery.delegationScheduledRequests(baltathar.address);
        const roundDelay = psConst.revokeDelegationDelay.toNumber();

        expect(delegationRequests[0].delegator.toString()).toBe(ethan.address);
        expect(delegationRequests[0].whenExecutable.toNumber()).toBe(currentRound + roundDelay);
        expect(delegationRequests[0].action.isDecrease).toBe(true);
        expect(delegationRequests[0].action.asDecrease.toNumber()).toBe(Number(LESS_AMOUNT));

        await createBlock(psTx.cancelDelegationRequest(baltathar.address).signAsync(ethan));

        const delegationRequestsAfterCancel = await psQuery.delegationScheduledRequests(
          baltathar.address
        );
        expect(delegationRequestsAfterCancel.isEmpty).toBe(true);
      },
    });
  },
});

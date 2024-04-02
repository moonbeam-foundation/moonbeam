import "@moonbeam-network/api-augment";
import { describeSuite, beforeAll, expect } from "@moonwall/cli";
import { MIN_GLMR_DELEGATOR, alith, baltathar, ethan } from "@moonwall/util";
import { jumpToRound } from "../../../../helpers/block.js";

describeSuite({
  id: "D013438",
  title: "Staking - Delegation Scheduled Requests with bondless collator - execute bond less early",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const LESS_AMOUNT = 10n;
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
      await createBlock(
        [
          sudo(psTx.setBlocksPerRound(10)).signAsync(alith),
          psTx.delegate(baltathar.address, MIN_GLMR_DELEGATOR + LESS_AMOUNT, 0, 0).signAsync(ethan),
        ],
        { allowFailures: false }
      );

      await createBlock(
        psTx.scheduleDelegatorBondLess(baltathar.address, LESS_AMOUNT).signAsync(ethan),
        { allowFailures: false }
      );

      // jump to a round before the actual executable Round
      const delegationRequests = await psQuery.delegationScheduledRequests(baltathar.address);
      await jumpToRound(context, delegationRequests[0].whenExecutable.toNumber() - 1);
    });

    it({
      id: "T01",
      title: "should fail",
      test: async () => {
        const block = await createBlock(
          psTx.executeDelegationRequest(ethan.address, baltathar.address).signAsync(ethan)
        );
        expect(block.result!.error!.name).to.equal("PendingDelegationRequestNotDueYet");
      },
    });
  },
});

import "@moonbeam-network/api-augment";
import { describeSuite, beforeAll, expect } from "@moonwall/cli";
import { MIN_GLMR_DELEGATOR, alith, baltathar, ethan } from "@moonwall/util";

describeSuite({
  id: "D2996",
  title: "Staking - Delegation Scheduled Requests with bondless collator - schedule bond less",
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
    });

    it({
      id: "T01",
      title: "should succeed",
      test: async () => {
        // We rely on the relay chain block number for rounds clocktime.
        //
        // This value 'LastRelayChainBlockNumber' starts on 1000 after we create our first
        // para-block in this environment (inside beforeAll).
        //
        // When we create a second block, this behavior will naturally modify the round number
        // in +1 due to the checks between should_update() function of parachain staking pallet.
        //
        // Given this, we first create an extra block to go to round 2 directly, and prevent
        // mismatches while comparing 'whenExecutable' field between rounds 1 and 2.
        await context.createBlock();

        const currentRound = (await psQuery.round()).current.toNumber();

        await createBlock(
          psTx.scheduleDelegatorBondLess(baltathar.address, LESS_AMOUNT).signAsync(ethan),
          { allowFailures: false }
        );

        const delegationRequestsAfter = await psQuery.delegationScheduledRequests(
          baltathar.address
        );
        const roundDelay = psConst.revokeDelegationDelay.toNumber();
        expect(delegationRequestsAfter[0].delegator.toString()).toBe(ethan.address);
        expect(delegationRequestsAfter[0].whenExecutable.toNumber()).toBe(
          currentRound + roundDelay
        );
        expect(delegationRequestsAfter[0].action.isDecrease).toBe(true);
        expect(delegationRequestsAfter[0].action.asDecrease.toNumber()).toBe(Number(LESS_AMOUNT));
      },
    });
  },
});

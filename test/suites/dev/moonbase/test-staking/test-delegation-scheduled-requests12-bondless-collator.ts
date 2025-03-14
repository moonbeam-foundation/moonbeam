import "@moonbeam-network/api-augment";
import { describeSuite, beforeAll, expect } from "@moonwall/cli";
import { MIN_GLMR_DELEGATOR, alith, baltathar, ethan } from "@moonwall/util";
import { jumpToRound } from "../../../../helpers/block.js";

describeSuite({
  id: "D013424",
  title: "Staking - Delegation Scheduled Requests with bondless collator - collator leave",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let whenExecutable: number;
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
      await createBlock([
        sudo(psTx.setBlocksPerRound(10)).signAsync(alith),
        psTx
          .delegateWithAutoCompound(alith.address, MIN_GLMR_DELEGATOR + 10n, 0, 0, 0, 0)
          .signAsync(ethan),
      ]);

      await createBlock([
        psTx
          .delegateWithAutoCompound(baltathar.address, MIN_GLMR_DELEGATOR + 10n, 0, 0, 0, 1)
          .signAsync(ethan),
      ]);
      await createBlock([psTx.scheduleDelegatorBondLess(alith.address, 10n).signAsync(ethan)]);

      await createBlock([
        psTx.scheduleDelegatorBondLess(baltathar.address, 10n).signAsync(ethan),
        psTx.scheduleLeaveCandidates(2).signAsync(baltathar),
      ]);

      const currentRound = (await psQuery.round()).current.toNumber();
      const roundDelay = psConst.revokeDelegationDelay.toNumber();
      whenExecutable = currentRound + roundDelay;

      const collatorState = await psQuery.candidateInfo(baltathar.address);
      await jumpToRound(context, collatorState.unwrap().status.asLeaving.toNumber());
    });

    it({
      id: "T01",
      title: "should remove complete storage item",
      test: async () => {
        const delegationRequestsBefore = await psQuery.delegationScheduledRequests(
          baltathar.address
        );
        expect(delegationRequestsBefore.toJSON()).to.not.be.empty;

        await createBlock(psTx.executeLeaveCandidates(baltathar.address, 1).signAsync(ethan));

        const delegationRequestsBaltatharAfter = await psQuery.delegationScheduledRequests(
          baltathar.address
        );
        const delegationRequestsAlithAfter = await psQuery.delegationScheduledRequests(
          alith.address
        );
        expect(delegationRequestsAlithAfter.toJSON()).to.deep.equal([
          {
            delegator: ethan.address,
            whenExecutable,
            action: {
              decrease: 10,
            },
          },
        ]);
        expect(delegationRequestsBaltatharAfter.toJSON()).to.be.empty;
        const delagationRequestsKeys = await psQuery.delegationScheduledRequests.keys();
        expect(delagationRequestsKeys.map((k) => k.args[0].toString())).to.deep.equal([
          alith.address,
        ]);
      },
    });
  },
});

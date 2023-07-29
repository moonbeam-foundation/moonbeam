import "@moonbeam-network/api-augment";
import { describeSuite, beforeAll, expect } from "@moonwall/cli";
import { MIN_GLMR_DELEGATOR, MIN_GLMR_STAKING, alith, baltathar, ethan } from "@moonwall/util";
import { jumpToRound } from "../../../helpers/block.js";

describeSuite({
  id: "D2932",
  title: "Staking - Delegation Scheduled Requests - execute bond less after round delay",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const LESS_AMOUNT = 10n;

    beforeAll(async () => {
      await context.createBlock(
        [
          context
            .polkadotJs()
            .tx.sudo.sudo(context.polkadotJs().tx.parachainStaking.setBlocksPerRound(10))
            .signAsync(alith),
          context
            .polkadotJs()
            .tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, 1)
            .signAsync(baltathar),
          context
            .polkadotJs()
            .tx.parachainStaking.delegate(alith.address, MIN_GLMR_DELEGATOR + LESS_AMOUNT, 0, 0)
            .signAsync(ethan),
        ],
        { allowFailures: false }
      );
      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.delegate(baltathar.address, MIN_GLMR_DELEGATOR + LESS_AMOUNT, 0, 1)
          .signAsync(ethan),
        { allowFailures: false }
      );
      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.scheduleDelegatorBondLess(alith.address, LESS_AMOUNT)
          .signAsync(ethan),
        { allowFailures: false }
      );

      // jump to exact executable Round
      const delegationRequests = await context
        .polkadotJs()
        .query.parachainStaking.delegationScheduledRequests(alith.address);
      await jumpToRound(context, delegationRequests[0].whenExecutable.toNumber() + 5);
    });

    it({
      id: "T01",
      title: "should succeed",
      test: async () => {
        await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.executeDelegationRequest(ethan.address, alith.address)
            .signAsync(ethan)
        );
        const delegatorState = await context
          .polkadotJs()
          .query.parachainStaking.delegatorState(ethan.address);
        const delegationRequestsAfter = await context
          .polkadotJs()
          .query.parachainStaking.delegationScheduledRequests(alith.address);
        expect(delegatorState.unwrap().delegations[0].owner.toString()).toBe(baltathar.address);
        expect(delegatorState.unwrap().delegations[0].amount.toBigInt()).toBe(
          MIN_GLMR_DELEGATOR + LESS_AMOUNT
        );
        expect(delegatorState.unwrap().delegations[1].owner.toString()).toBe(alith.address);
        expect(delegatorState.unwrap().delegations[1].amount.toBigInt()).toBe(MIN_GLMR_DELEGATOR);
        expect(delegationRequestsAfter.isEmpty).toBe(true);
      },
    });
  },
});

// describeSuite({id:"",title:id:"",title:"Staking - Delegation Scheduled Requests - collator leave",foundationMethods:"dev",testCases: ({context,it,log}) => {
//   let whenExecutable: number;
//   before("should delegate and add baltathar as candidate", test: async () => {
//     await expectOk(
//       context.createBlock([
//         context.polkadotJs().tx.sudo
//           .sudo(context.polkadotJs().tx.parachainStaking.setBlocksPerRound(10))
//           .signAsync(alith),
//         context.polkadotJs().tx.parachainStaking
//           .joinCandidates(MIN_GLMR_STAKING, 1)
//           .signAsync(baltathar),
//         context.polkadotJs().tx.parachainStaking
//           .delegate(alith.address, MIN_GLMR_DELEGATOR + 10n, 0, 0)
//           .signAsync(ethan),
//       ])
//     );

//     await expectOk(
//       context.createBlock([
//         context.polkadotJs().tx.parachainStaking
//           .delegate(baltathar.address, MIN_GLMR_DELEGATOR + 10n, 0, 1)
//           .signAsync(ethan),
//       ])
//     );
//     await expectOk(
//       context.createBlock([
//         context.polkadotJs().tx.parachainStaking
//           .scheduleDelegatorBondLess(alith.address, 10n)
//           .signAsync(ethan),
//       ])
//     );

//     await expectOk(
//       context.createBlock([
//         context.polkadotJs().tx.parachainStaking
//           .scheduleDelegatorBondLess(baltathar.address, 10n)
//           .signAsync(ethan),
//         context.polkadotJs().tx.parachainStaking.scheduleLeaveCandidates(2).signAsync(baltathar),
//       ])
//     );

//     const currentRound = (
//       await context.polkadotJs().query.parachainStaking.round()
//     ).current.toNumber();
//     const roundDelay = context.polkadotJs().consts.parachainStaking.revokeDelegationDelay.toNumber();
//     whenExecutable = currentRound + roundDelay;

//     const collatorState = await context.polkadotJs().query.parachainStaking.candidateInfo(
//       baltathar.address
//     );
//     await jumpToRound(context, collatorState.unwrap().status.asLeaving.toNumber());
//   });

//   it({id:"",title:"should remove complete storage item", test: async () => {
//     const delegationRequestsBefore =
//       await context.polkadotJs().query.parachainStaking.delegationScheduledRequests(
//         baltathar.address
//       );
//     expect(delegationRequestsBefore.toJSON()).to.not.be.empty;

//     await context.createBlock(
//       context.polkadotJs().tx.parachainStaking
//         .executeLeaveCandidates(baltathar.address, 1)
//         .signAsync(ethan)
//     );

//     const delegationRequestsBaltatharAfter =
//       await context.polkadotJs().query.parachainStaking.delegationScheduledRequests(
//         baltathar.address
//       );
//     const delegationRequestsAlithAfter =
//       await context.polkadotJs().query.parachainStaking.delegationScheduledRequests(alith.address);
//     expect(delegationRequestsAlithAfter.toJSON()).to.deep.equal([
//       {
//         delegator: ethan.address,
//         whenExecutable,
//         action: {
//           decrease: 10,
//         },
//       },
//     ]);
//     expect(delegationRequestsBaltatharAfter.toJSON()).to.be.empty;
//     const delagationRequestsKeys =
//       await context.polkadotJs().query.parachainStaking.delegationScheduledRequests.keys();
//     expect(delagationRequestsKeys.map((k) => k.args[0].toString())).to.deep.equal([alith.address]);
//   });
// });

// describeSuite({id:"",title:id:"",title:"Staking - Delegation Scheduled Requests - delegator leave",foundationMethods:"dev",testCases: ({context,it,log}) => {
//   before("should delegate and add baltathar as candidate", test: async () => {
//     await expectOk(
//       context.createBlock([
//         context.polkadotJs().tx.sudo
//           .sudo(context.polkadotJs().tx.parachainStaking.setBlocksPerRound(10))
//           .signAsync(alith),
//         context.polkadotJs().tx.parachainStaking
//           .joinCandidates(MIN_GLMR_STAKING, 1)
//           .signAsync(baltathar),
//         context.polkadotJs().tx.parachainStaking
//           .delegate(alith.address, MIN_GLMR_DELEGATOR + 10n, 0, 0)
//           .signAsync(ethan),
//       ])
//     );
//     await expectOk(
//       context.createBlock([
//         context.polkadotJs().tx.parachainStaking
//           .delegate(baltathar.address, MIN_GLMR_DELEGATOR + 10n, 0, 1)
//           .signAsync(ethan),
//       ])
//     );
//     await expectOk(
//       context.createBlock([
//         context.polkadotJs().tx.parachainStaking
//           .scheduleDelegatorBondLess(alith.address, 10n)
//           .signAsync(ethan),
//       ])
//     );
//     await expectOk(
//       context.createBlock(
//         context.polkadotJs().tx.parachainStaking
//           .scheduleDelegatorBondLess(baltathar.address, 10n)
//           .signAsync(ethan)
//       )
//     );
//     await expectOk(
//       context.createBlock(
//         context.polkadotJs().tx.parachainStaking.scheduleLeaveDelegators().signAsync(ethan)
//       )
//     );

//     const roundDelay = context.polkadotJs().consts.parachainStaking.leaveDelegatorsDelay.toNumber();
//     await jumpRounds(context, roundDelay);
//   });

//   it({id:"",title:"should remove complete scheduled requests across multiple candidates", test: async () => {
//     const delegationRequestsAlithBefore =
//       await context.polkadotJs().query.parachainStaking.delegationScheduledRequests(alith.address);
//     const delegationRequestsBaltatharBefore =
//       await context.polkadotJs().query.parachainStaking.delegationScheduledRequests(
//         baltathar.address
//       );
//     expect(delegationRequestsAlithBefore.toJSON()).to.not.be.empty;
//     expect(delegationRequestsBaltatharBefore.toJSON()).to.not.be.empty;

//     await context.createBlock(
//       context.polkadotJs().tx.parachainStaking
//         .executeLeaveDelegators(ethan.address, 2)
//         .signAsync(ethan)
//     );

//     const delegationRequestsAlithAfter =
//       await context.polkadotJs().query.parachainStaking.delegationScheduledRequests(alith.address);
//     const delegationRequestsBaltatharAfter =
//       await context.polkadotJs().query.parachainStaking.delegationScheduledRequests(
//         baltathar.address
//       );
//     expect(delegationRequestsAlithAfter.toJSON()).to.be.empty;
//     expect(delegationRequestsBaltatharAfter.toJSON()).to.be.empty;
//   });
// });
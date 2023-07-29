import "@moonbeam-network/api-augment";
import { describeSuite, beforeAll, expect } from "@moonwall/cli";
import { MIN_GLMR_DELEGATOR, MIN_GLMR_STAKING, alith, baltathar, ethan } from "@moonwall/util";
import { jumpToRound } from "../../../helpers/block.js";

describeSuite({
  id: "D2929",
  title: "Staking - Delegation Scheduled Requests - cancel scheduled bond less",
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
            .tx.parachainStaking.delegate(alith.address, MIN_GLMR_DELEGATOR + LESS_AMOUNT, 0, 0)
            .signAsync(ethan),
        ],
        { allowFailures: false }
      );

      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.scheduleDelegatorBondLess(alith.address, LESS_AMOUNT)
          .signAsync(ethan)
      );
    });

    it({
      id: "T01",
      title: "should succeed",
      test: async () => {
        const currentRound = (
          await context.polkadotJs().query.parachainStaking.round()
        ).current.toNumber();
        const delegationRequests = await context
          .polkadotJs()
          .query.parachainStaking.delegationScheduledRequests(alith.address);
        const roundDelay = context
          .polkadotJs()
          .consts.parachainStaking.revokeDelegationDelay.toNumber();

        expect(delegationRequests[0].delegator.toString()).toBe(ethan.address);
        expect(delegationRequests[0].whenExecutable.toNumber()).toBe(currentRound + roundDelay);
        expect(delegationRequests[0].action.isDecrease).toBe(true);
        expect(delegationRequests[0].action.asDecrease.toNumber()).toBe(Number(LESS_AMOUNT));

        await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.cancelDelegationRequest(alith.address)
            .signAsync(ethan)
        );

        const delegationRequestsAfterCancel = await context
          .polkadotJs()
          .query.parachainStaking.delegationScheduledRequests(alith.address);
        expect(delegationRequestsAfterCancel.isEmpty).toBe(true);
      },
    });
  },
});

// describeSuite({id:"",title:
//   "Staking - Delegation Scheduled Requests - execute bond less early",
//   (context) => {
//     const LESS_AMOUNT = 10n;

//     before("should schedule bond less and jump to premature round", test: async () => {
//       await expectOk(
//         context.createBlock([
//           context.polkadotJs().tx.sudo
//             .sudo(context.polkadotJs().tx.parachainStaking.setBlocksPerRound(10))
//             .signAsync(alith),
//           context.polkadotJs().tx.parachainStaking
//             .delegate(alith.address, MIN_GLMR_DELEGATOR + LESS_AMOUNT, 0, 0)
//             .signAsync(ethan),
//         ])
//       );

//       await expectOk(
//         context.createBlock(
//           context.polkadotJs().tx.parachainStaking
//             .scheduleDelegatorBondLess(alith.address, LESS_AMOUNT)
//             .signAsync(ethan)
//         )
//       );

//       // jump to a round before the actual executable Round
//       const delegationRequests =
//         await context.polkadotJs().query.parachainStaking.delegationScheduledRequests(alith.address);
//       await jumpToRound(context, delegationRequests[0].whenExecutable.toNumber() - 1);
//     });

//     it({id:"",title:"should fail", test: async () => {
//       const block = await context.createBlock(
//         context.polkadotJs().tx.parachainStaking
//           .executeDelegationRequest(ethan.address, alith.address)
//           .signAsync(ethan)
//       );
//       expect(block.result.error.name).to.equal("PendingDelegationRequestNotDueYet");
//     });
//   }
// );

// describeSuite({id:"",title:
//   "Staking - Delegation Scheduled Requests -execute bond less exact round",
//   (context) => {
//     const LESS_AMOUNT = 10n;

//     before("should schedule bond less and jump to exact round", test: async () => {
//       await expectOk(
//         context.createBlock(
//           context.polkadotJs().tx.parachainStaking
//             .joinCandidates(MIN_GLMR_STAKING, 1)
//             .signAsync(baltathar)
//         )
//       );
//       await expectOk(
//         context.createBlock([
//           context.polkadotJs().tx.sudo
//             .sudo(context.polkadotJs().tx.parachainStaking.setBlocksPerRound(10))
//             .signAsync(alith),
//           context.polkadotJs().tx.parachainStaking
//             .delegate(alith.address, MIN_GLMR_DELEGATOR + LESS_AMOUNT, 0, 0)
//             .signAsync(ethan),
//         ])
//       );
//       await expectOk(
//         context.createBlock(
//           context.polkadotJs().tx.parachainStaking
//             .delegate(baltathar.address, MIN_GLMR_DELEGATOR + LESS_AMOUNT, 0, 1)
//             .signAsync(ethan)
//         )
//       );
//       await expectOk(
//         context.createBlock(
//           context.polkadotJs().tx.parachainStaking
//             .scheduleDelegatorBondLess(alith.address, LESS_AMOUNT)
//             .signAsync(ethan)
//         )
//       );

//       // jump to exact executable Round
//       const delegationRequests =
//         await context.polkadotJs().query.parachainStaking.delegationScheduledRequests(alith.address);
//       await jumpToRound(context, delegationRequests[0].whenExecutable.toNumber());
//     });

//     it({id:"",title:"should succeed", test: async () => {
//       await context.createBlock(
//         context.polkadotJs().tx.parachainStaking
//           .executeDelegationRequest(ethan.address, alith.address)
//           .signAsync(ethan)
//       );
//       const delegatorState = await context.polkadotJs().query.parachainStaking.delegatorState(
//         ethan.address
//       );
//       const delegationRequestsAfter =
//         await context.polkadotJs().query.parachainStaking.delegationScheduledRequests(alith.address);
//       expect(delegatorState.unwrap().delegations.toJSON()).to.deep.equal([
//         {
//           owner: baltathar.address,
//           amount: numberToHex(MIN_GLMR_DELEGATOR + LESS_AMOUNT),
//         },
//         {
//           owner: alith.address,
//           amount: numberToHex(MIN_GLMR_DELEGATOR),
//         },
//       ]);
//       expect(delegationRequestsAfter.toJSON()).to.be.empty;
//     });
//   }
// );

// describeSuite({id:"",title:
//   "Staking - Delegation Scheduled Requests - execute bond less after round delay",
//   (context) => {
//     const LESS_AMOUNT = 10n;

//     before("should schedule bond less and jump to after round", test: async () => {
//       await expectOk(
//         context.createBlock([
//           context.polkadotJs().tx.sudo
//             .sudo(context.polkadotJs().tx.parachainStaking.setBlocksPerRound(10))
//             .signAsync(alith),
//           context.polkadotJs().tx.parachainStaking
//             .joinCandidates(MIN_GLMR_STAKING, 1)
//             .signAsync(baltathar),
//           context.polkadotJs().tx.parachainStaking
//             .delegate(alith.address, MIN_GLMR_DELEGATOR + LESS_AMOUNT, 0, 0)
//             .signAsync(ethan),
//         ])
//       );
//       await expectOk(
//         context.createBlock(
//           context.polkadotJs().tx.parachainStaking
//             .delegate(baltathar.address, MIN_GLMR_DELEGATOR + LESS_AMOUNT, 0, 1)
//             .signAsync(ethan)
//         )
//       );
//       await expectOk(
//         context.createBlock(
//           context.polkadotJs().tx.parachainStaking
//             .scheduleDelegatorBondLess(alith.address, LESS_AMOUNT)
//             .signAsync(ethan)
//         )
//       );

//       // jump to exact executable Round
//       const delegationRequests =
//         await context.polkadotJs().query.parachainStaking.delegationScheduledRequests(alith.address);
//       await jumpToRound(context, delegationRequests[0].whenExecutable.toNumber() + 5);
//     });

//     it({id:"",title:"should succeed", test: async () => {
//       await context.createBlock(
//         context.polkadotJs().tx.parachainStaking
//           .executeDelegationRequest(ethan.address, alith.address)
//           .signAsync(ethan)
//       );
//       const delegatorState = await context.polkadotJs().query.parachainStaking.delegatorState(
//         ethan.address
//       );
//       const delegationRequestsAfter =
//         await context.polkadotJs().query.parachainStaking.delegationScheduledRequests(alith.address);
//       expect(delegatorState.unwrap().delegations.toJSON()).to.deep.equal([
//         {
//           owner: baltathar.address,
//           amount: numberToHex(MIN_GLMR_DELEGATOR + LESS_AMOUNT),
//         },
//         {
//           owner: alith.address,
//           amount: numberToHex(MIN_GLMR_DELEGATOR),
//         },
//       ]);
//       expect(delegationRequestsAfter.toJSON()).to.be.empty;
//     });
//   }
// );

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
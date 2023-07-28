import "@moonbeam-network/api-augment";
import { describeSuite, beforeAll, expect } from "@moonwall/cli";
import { MIN_GLMR_DELEGATOR, alith, ethan } from "@moonwall/util";

const numberToHex = (n: BigInt): string => `0x${n.toString(16).padStart(32, "0")}`;

describeSuite({
  id: "D2923",
  title: "Staking - Delegation Scheduled Requests - cancel scheduled revoke",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      await context.createBlock([
        context
          .polkadotJs()
          .tx.sudo.sudo(context.polkadotJs().tx.parachainStaking.setBlocksPerRound(10))
          .signAsync(alith),
        context
          .polkadotJs()
          .tx.parachainStaking.delegate(alith.address, MIN_GLMR_DELEGATOR, 0, 0)
          .signAsync(ethan),
      ]);

      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.scheduleRevokeDelegation(alith.address)
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
        const delegationRequestsAfterSchedule = await context
          .polkadotJs()
          .query.parachainStaking.delegationScheduledRequests(alith.address);
        const roundDelay = context
          .polkadotJs()
          .consts.parachainStaking.revokeDelegationDelay.toNumber();
        expect(delegationRequestsAfterSchedule.toJSON()).to.deep.equal([
          {
            delegator: ethan.address,
            whenExecutable: currentRound + roundDelay,
            action: {
              revoke: numberToHex(MIN_GLMR_DELEGATOR),
            },
          },
        ]);

        await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.cancelDelegationRequest(alith.address)
            .signAsync(ethan)
        );

        const delegationRequestsAfterCancel = await context
          .polkadotJs()
          .query.parachainStaking.delegationScheduledRequests(alith.address);
        expect(delegationRequestsAfterCancel).to.be.empty;
      },
    });
  },
});

// describeSuite({id:"",title:"Staking - Delegation Scheduled Requests - execute revoke early",foundationMethods:"dev",testCases: ({context,it,log}) => {
//   before("should schedule revoke and jump to early round", test: async () => {
//     await expectOk(
//       context.createBlock([
//         context.polkadotJs().tx.sudo
//           .sudo(context.polkadotJs().tx.parachainStaking.setBlocksPerRound(10))
//           .signAsync(alith),
//         context.polkadotJs().tx.parachainStaking
//           .delegate(alith.address, MIN_GLMR_DELEGATOR, 0, 0)
//           .signAsync(ethan),
//       ])
//     );

//     await expectOk(
//       context.createBlock(
//         context.polkadotJs().tx.parachainStaking
//           .scheduleRevokeDelegation(alith.address)
//           .signAsync(ethan)
//       )
//     );

//     // jump to a round before the actual executable Round
//     const delegationRequests =
//       await context.polkadotJs().query.parachainStaking.delegationScheduledRequests(alith.address);
//     await jumpToRound(context, delegationRequests[0].whenExecutable.toNumber() - 1);
//   });

//   it({id:"",title:"should fail", test: async () => {
//     const block = await context.createBlock(
//       context.polkadotJs().tx.parachainStaking
//         .executeDelegationRequest(ethan.address, alith.address)
//         .signAsync(ethan)
//     );
//     expect(block.result.error.name).to.equal("PendingDelegationRequestNotDueYet");
//   });
// });

// describeDevMoonbeam(
//   "Staking - Delegation Scheduled Requests - execute revoke exact round delay",
//   (context) => {
//     before("should schedule revoke and jump to exact round", test: async () => {
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
//             .delegate(alith.address, MIN_GLMR_DELEGATOR, 0, 0)
//             .signAsync(ethan),
//         ])
//       );
//       await expectOk(
//         context.createBlock(
//           context.polkadotJs().tx.parachainStaking
//             .delegate(baltathar.address, MIN_GLMR_DELEGATOR, 0, 1)
//             .signAsync(ethan)
//         )
//       );
//       await expectOk(
//         context.createBlock(
//           context.polkadotJs().tx.parachainStaking
//             .scheduleRevokeDelegation(alith.address)
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
//           amount: numberToHex(MIN_GLMR_DELEGATOR),
//         },
//       ]);
//       expect(delegationRequestsAfter.toJSON()).to.be.empty;
//     });
//   }
// );

// describeDevMoonbeam(
//   "Staking - Delegation Scheduled Requests - execute revoke after round delay",
//   (context) => {
//     before("should schedule revoke and jump to after round", test: async () => {
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
//             .delegate(alith.address, MIN_GLMR_DELEGATOR, 0, 0)
//             .signAsync(ethan),
//         ])
//       );
//       await expectOk(
//         context.createBlock(
//           context.polkadotJs().tx.parachainStaking
//             .delegate(baltathar.address, MIN_GLMR_DELEGATOR, 0, 1)
//             .signAsync(ethan)
//         )
//       );
//       await expectOk(
//         context.createBlock(
//           context.polkadotJs().tx.parachainStaking
//             .scheduleRevokeDelegation(alith.address)
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
//           amount: numberToHex(MIN_GLMR_DELEGATOR),
//         },
//       ]);
//       expect(delegationRequestsAfter.toJSON()).to.be.empty;
//     });
//   }
// );

// describeDevMoonbeam(
//   "Staking - Delegation Scheduled Requests - execute revoke on last delegation",
//   (context) => {
//     before("should schedule revoke and jump to exact round", test: async () => {
//       await expectOk(
//         context.createBlock([
//           context.polkadotJs().tx.sudo
//             .sudo(context.polkadotJs().tx.parachainStaking.setBlocksPerRound(10))
//             .signAsync(alith),
//           context.polkadotJs().tx.parachainStaking
//             .delegate(alith.address, MIN_GLMR_DELEGATOR, 0, 0)
//             .signAsync(ethan),
//         ])
//       );

//       await expectOk(
//         context.createBlock(
//           context.polkadotJs().tx.parachainStaking
//             .scheduleRevokeDelegation(alith.address)
//             .signAsync(ethan)
//         )
//       );

//       // jump to exact executable Round
//       const delegationRequests =
//         await context.polkadotJs().query.parachainStaking.delegationScheduledRequests(alith.address);
//       await jumpToRound(context, delegationRequests[0].whenExecutable.toNumber());
//     });

//     it({id:"",title:"should succeed and leave as delegator", test: async () => {
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
//       expect(delegatorState.isNone).to.be.true; // last delegation revoked, so delegator left
//       expect(delegationRequestsAfter.toJSON()).to.be.empty;
//     });
//   }
// );

// describeSuite({id:"",title:"Staking - Delegation Scheduled Requests - schedule bond less",foundationMethods:"dev",testCases: ({context,it,log}) => {
//   const LESS_AMOUNT = 10n;

//   before("should delegate", test: async () => {
//     await expectOk(
//       context.createBlock([
//         context.polkadotJs().tx.sudo
//           .sudo(context.polkadotJs().tx.parachainStaking.setBlocksPerRound(10))
//           .signAsync(alith),
//         context.polkadotJs().tx.parachainStaking
//           .delegate(alith.address, MIN_GLMR_DELEGATOR + LESS_AMOUNT, 0, 0)
//           .signAsync(ethan),
//       ])
//     );
//   });

//   it({id:"",title:"should succeed", test: async () => {
//     const currentRound = (
//       await context.polkadotJs().query.parachainStaking.round()
//     ).current.toNumber();

//     await context.createBlock(
//       context.polkadotJs().tx.parachainStaking
//         .scheduleDelegatorBondLess(alith.address, LESS_AMOUNT)
//         .signAsync(ethan)
//     );

//     const delegationRequestsAfter =
//       await context.polkadotJs().query.parachainStaking.delegationScheduledRequests(alith.address);
//     const roundDelay = context.polkadotJs().consts.parachainStaking.revokeDelegationDelay.toNumber();
//     expect(delegationRequestsAfter.toJSON()).to.deep.equal([
//       {
//         delegator: ethan.address,
//         whenExecutable: currentRound + roundDelay,
//         action: {
//           decrease: Number(LESS_AMOUNT),
//         },
//       },
//     ]);
//   });
// });

// describeDevMoonbeam(
//   "Staking - Delegation Scheduled Requests - cancel scheduled bond less",
//   (context) => {
//     const LESS_AMOUNT = 10n;

//     before("should delegate", test: async () => {
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
//     });

//     it({id:"",title:"should succeed", test: async () => {
//       const currentRound = (
//         await context.polkadotJs().query.parachainStaking.round()
//       ).current.toNumber();
//       const delegationRequests =
//         await context.polkadotJs().query.parachainStaking.delegationScheduledRequests(alith.address);
//       const roundDelay =
//         context.polkadotJs().consts.parachainStaking.revokeDelegationDelay.toNumber();
//       expect(delegationRequests.toJSON()).to.deep.equal([
//         {
//           delegator: ethan.address,
//           whenExecutable: currentRound + roundDelay,
//           action: {
//             decrease: Number(LESS_AMOUNT),
//           },
//         },
//       ]);

//       await context.createBlock(
//         context.polkadotJs().tx.parachainStaking
//           .cancelDelegationRequest(alith.address)
//           .signAsync(ethan)
//       );

//       const delegationRequestsAfterCancel =
//         await context.polkadotJs().query.parachainStaking.delegationScheduledRequests(alith.address);
//       expect(delegationRequestsAfterCancel.toJSON()).to.be.empty;
//     });
//   }
// );

// describeDevMoonbeam(
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

// describeDevMoonbeam(
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

// describeDevMoonbeam(
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

// describeSuite({id:"",title:"Staking - Delegation Scheduled Requests - collator leave",foundationMethods:"dev",testCases: ({context,it,log}) => {
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

// describeSuite({id:"",title:"Staking - Delegation Scheduled Requests - delegator leave",foundationMethods:"dev",testCases: ({context,it,log}) => {
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

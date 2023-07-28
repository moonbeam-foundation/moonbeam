import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { MIN_GLMR_STAKING, alith, ethan } from "@moonwall/util";

describeSuite({
  id: "D2904",
  title: "Staking - Candidate Leave Schedule - hint too low",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, 1)
          .signAsync(ethan),
        { signer: alith, allowFailures: false }
      );
    });

    it({
      id: "T01",
      title: "should fail",
      test: async () => {
        const block = await context.createBlock(
          context.polkadotJs().tx.parachainStaking.scheduleLeaveCandidates(1).signAsync(ethan)
        );
        expect(block.result!.successful).to.be.false;
        expect(block.result!.error!.name).to.equal("TooLowCandidateCountToLeaveCandidates");
      },
    });
  },
});

// describeSuite({id:"",title:"Staking - Candidate Leave Schedule - already scheduled",foundationMethods:"dev",testCases: ({context,it,log}) => {
//   before("should join candidate", async () => {
//     await expectOk(
//       context.createBlock(
//         context.polkadotJs().tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, 1).signAsync(ethan)
//       )
//     );
//     await expectOk(
//       context.createBlock(
//         context.polkadotJs().tx.parachainStaking.scheduleLeaveCandidates(2).signAsync(ethan)
//       )
//     );
//   });

//   it("should fail", async () => {
//     const block = await context.createBlock(
//       context.polkadotJs().tx.parachainStaking.scheduleLeaveCandidates(2).signAsync(ethan)
//     );
//     expect(block.result.successful).to.be.false;
//     expect(block.result.error.name).to.equal("CandidateAlreadyLeaving");
//   });
// });

// describeSuite({id:"",title:"Staking - Candidate Leave Schedule - valid request",foundationMethods:"dev",testCases: ({context,it,log}) => {
//   before("should join candidate", async () => {
//     await expectOk(
//       context.createBlock(
//         context.polkadotJs().tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, 1).signAsync(ethan)
//       )
//     );
//     await expectOk(
//       context.createBlock(
//         context.polkadotJs().tx.parachainStaking.scheduleLeaveCandidates(2).signAsync(ethan)
//       )
//     );
//   });

//   it("should change status to leaving at correct round", async () => {
//     const candidatePool = (await context.polkadotJs().query.parachainStaking.candidatePool()).map(
//       (c) => c.owner.toString()
//     );
//     const candidateState = (
//       await context.polkadotJs().query.parachainStaking.candidateInfo(ethan.address)
//     ).unwrap();
//     const leaveDelay = context.polkadotJs().consts.parachainStaking.leaveCandidatesDelay;
//     const currentRound = (await context.polkadotJs().query.parachainStaking.round()).current;

//     expect(candidatePool).to.be.deep.equal([alith.address]);
//     expect(candidateState.status.isLeaving).to.be.true;
//     expect(candidateState.status.asLeaving.toNumber()).to.equal(
//       currentRound.add(leaveDelay).toNumber()
//     );
//   });
// });

// describeSuite({id:"",title:"Staking - Candidate Leave Execute - before round delay",foundationMethods:"dev",testCases: ({context,it,log}) => {
//   before("should join candidates, schedule leave, and jump to earlier round", async () => {
//     await expectOk(
//       context.createBlock([
//         context.polkadotJs().tx.sudo
//           .sudo(context.polkadotJs().tx.parachainStaking.setBlocksPerRound(10))
//           .signAsync(alith),
//         context.polkadotJs().tx.parachainStaking
//           .joinCandidates(MIN_GLMR_STAKING, 1)
//           .signAsync(ethan),
//       ])
//     );
//     await expectOk(
//       context.createBlock(
//         context.polkadotJs().tx.parachainStaking.scheduleLeaveCandidates(2).signAsync(ethan)
//       )
//     );

//     const leaveDelay = context.polkadotJs().consts.parachainStaking.leaveDelegatorsDelay;
//     await jumpRounds(context, leaveDelay.subn(1).toNumber());
//   });

//   it("should fail", async () => {
//     const block = await context.createBlock(
//       context.polkadotJs().tx.parachainStaking
//         .executeLeaveCandidates(ethan.address, 0)
//         .signAsync(ethan)
//     );
//     expect(block.result.successful).to.be.false;
//     expect(block.result.error.name).to.equal("CandidateCannotLeaveYet");
//   });
// });

// describeSuite({id:"",title:"Staking - Candidate Leave Execute - exact round delay",foundationMethods:"dev",testCases: ({context,it,log}) => {
//   before("should join candidates, schedule leave, and jump to exact round", async () => {
//     await expectOk(
//       context.createBlock([
//         context.polkadotJs().tx.sudo
//           .sudo(context.polkadotJs().tx.parachainStaking.setBlocksPerRound(10))
//           .signAsync(alith),
//         context.polkadotJs().tx.parachainStaking
//           .joinCandidates(MIN_GLMR_STAKING, 1)
//           .signAsync(ethan),
//       ])
//     );
//     await expectOk(
//       context.createBlock(
//         context.polkadotJs().tx.parachainStaking.scheduleLeaveCandidates(2).signAsync(ethan)
//       )
//     );
//     const leaveDelay = context.polkadotJs().consts.parachainStaking.leaveDelegatorsDelay;
//     await jumpRounds(context, leaveDelay.toNumber());
//   });

//   it("should succeed", async () => {
//     const block = await context.createBlock(
//       context.polkadotJs().tx.parachainStaking
//         .executeLeaveCandidates(ethan.address, 0)
//         .signAsync(ethan)
//     );
//     expect(block.result.successful).to.be.true;
//     const leaveEvents = block.result.events.reduce((acc, event) => {
//       if (context.polkadotJs().events.parachainStaking.CandidateLeft.is(event.event)) {
//         acc.push({
//           account: event.event.data[0].toString(),
//         });
//       }
//       return acc;
//     }, []);

//     expect(leaveEvents).to.deep.equal([
//       {
//         account: ethan.address,
//       },
//     ]);
//   });
// });

// describeSuite({id:"",title:"Staking - Candidate Leave Execute - after round delay",foundationMethods:"dev",testCases: ({context,it,log}) => {
//   before("should join candidates, schedule leave, and jump to after round", async () => {
//     await expectOk(
//       context.createBlock([
//         context.polkadotJs().tx.sudo
//           .sudo(context.polkadotJs().tx.parachainStaking.setBlocksPerRound(10))
//           .signAsync(alith),
//         context.polkadotJs().tx.parachainStaking
//           .joinCandidates(MIN_GLMR_STAKING, 1)
//           .signAsync(ethan),
//       ])
//     );

//     await expectOk(
//       context.createBlock(
//         context.polkadotJs().tx.parachainStaking.scheduleLeaveCandidates(2).signAsync(ethan)
//       )
//     );

//     const leaveDelay = context.polkadotJs().consts.parachainStaking.leaveDelegatorsDelay;
//     await jumpRounds(context, leaveDelay.addn(5).toNumber());
//   });

//   it("should succeed", async () => {
//     const block = await context.createBlock(
//       context.polkadotJs().tx.parachainStaking
//         .executeLeaveCandidates(ethan.address, 0)
//         .signAsync(ethan)
//     );
//     expect(block.result.successful).to.be.true;
//     const leaveEvents = block.result.events.reduce((acc, event) => {
//       if (context.polkadotJs().events.parachainStaking.CandidateLeft.is(event.event)) {
//         acc.push({
//           account: event.event.data[0].toString(),
//         });
//       }
//       return acc;
//     }, []);

//     expect(leaveEvents).to.deep.equal([
//       {
//         account: ethan.address,
//       },
//     ]);
//   });
// });

// describeSuite({id:"",title:"Staking - Candidate Leave Cancel - leave not scheduled",foundationMethods:"dev",testCases: ({context,it,log}) => {
//   before("should join candidates", async () => {
//     await expectOk(
//       context.createBlock(
//         context.polkadotJs().tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, 1).signAsync(ethan)
//       )
//     );
//   });

//   it("should fail", async () => {
//     const block = await context.createBlock(
//       context.polkadotJs().tx.parachainStaking.cancelLeaveCandidates(2).signAsync(ethan)
//     );
//     expect(block.result.error.name).to.equal("CandidateNotLeaving");
//   });
// });

// describeSuite({id:"",title:"Staking - Candidate Leave Cancel - leave scheduled",foundationMethods:"dev",testCases: ({context,it,log}) => {
//   before("should join candidates and schedule leave", async () => {
//     await expectOk(
//       context.createBlock(
//         context.polkadotJs().tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, 1).signAsync(ethan)
//       )
//     );
//     await expectOk(
//       context.createBlock(
//         context.polkadotJs().tx.parachainStaking.scheduleLeaveCandidates(2).signAsync(ethan)
//       )
//     );
//   });

//   it("should succeed", async () => {
//     const candidateStateBefore = (
//       await context.polkadotJs().query.parachainStaking.candidateInfo(ethan.address)
//     ).unwrap();
//     expect(candidateStateBefore.status.isLeaving).to.be.true;

//     const block = await context.createBlock(
//       context.polkadotJs().tx.parachainStaking.cancelLeaveCandidates(2).signAsync(ethan)
//     );
//     expect(block.result.successful).to.be.true;

//     const candidateStateAfter = (
//       await context.polkadotJs().query.parachainStaking.candidateInfo(ethan.address)
//     ).unwrap();
//     expect(candidateStateAfter.status.isActive).to.be.true;
//   });
// });

import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { MIN_GLMR_STAKING, alith, ethan } from "@moonwall/util";
import { jumpRounds } from "../../../helpers/block.js";

describeSuite({
  id: "D2909",
  title: "Staking - Candidate Leave Execute - after round delay",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
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
            .signAsync(ethan),
        ],
        { signer: alith, allowFailures: false }
      );

      await context.createBlock(
        context.polkadotJs().tx.parachainStaking.scheduleLeaveCandidates(2).signAsync(ethan),
        { signer: alith, allowFailures: false }
      );

      const leaveDelay = context.polkadotJs().consts.parachainStaking.leaveDelegatorsDelay;
      await jumpRounds(context, leaveDelay.addn(5).toNumber());
    });

    it({
      id: "T01",
      title: "should succeed",
      test: async () => {
        const block = await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.executeLeaveCandidates(ethan.address, 0)
            .signAsync(ethan)
        );
        expect(block.result!.successful).to.be.true;
        const leaveEvents = block.result!.events.reduce((acc, event) => {
          if (context.polkadotJs().events.parachainStaking.CandidateLeft.is(event.event)) {
            acc.push({
              account: event.event.data[0].toString(),
            });
          }
          return acc;
        }, []);

        expect(leaveEvents).to.deep.equal([
          {
            account: ethan.address,
          },
        ]);
      },
    });
  },
});

// describeSuite({id:"D2910",title:"Staking - Candidate Leave Cancel - leave not scheduled",foundationMethods:"dev",testCases: ({context,it,log}) => {
//   beforeAll("should join candidates", async () => {
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

// describeSuite({id:"D2911",title:"Staking - Candidate Leave Cancel - leave scheduled",foundationMethods:"dev",testCases: ({context,it,log}) => {
//   beforeAll("should join candidates and schedule leave", async () => {
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

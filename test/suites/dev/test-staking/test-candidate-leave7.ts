import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { MIN_GLMR_STAKING, alith, ethan } from "@moonwall/util";
import { jumpRounds } from "../../../helpers/block.js";

describeSuite({
  id: "D2910",
  title: "Staking - Candidate Leave Cancel - leave not scheduled",
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
          context.polkadotJs().tx.parachainStaking.cancelLeaveCandidates(2).signAsync(ethan)
        );
        expect(block.result!.error!.name).to.equal("CandidateNotLeaving");
      },
    });
  },
});

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

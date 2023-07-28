import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, MIN_GLMR_DELEGATOR, MIN_GLMR_STAKING, alith, ethan } from "@moonwall/util";

describeSuite({
  id: "D2902",
  title: "Staking - Candidate Join - already a delegator",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.delegate(ALITH_ADDRESS, MIN_GLMR_DELEGATOR, 0, 0)
          .signAsync(ethan),
        { signer: alith, allowFailures: false }
      );
    });

    it({
      id: "T01",
      title: "should fail",
      test: async () => {
        const block = await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, 1)
            .signAsync(ethan)
        );
        expect(block.result!.successful).to.be.false;
        expect(block.result!.error!.name).to.equal("DelegatorExists");
      },
    });
  },
});


// describeSuite({
//   id: "",
//   title: "Staking - Candidate Join - valid request",
//   foundationMethods: "dev",
//   testCases: ({ context, it, log }) => {
//     const numberToHex = (n: BigInt): string => `0x${n.toString(16).padStart(32, "0")}`;

//     beforeAll(async () => {
//       await expectOk(
//         context.createBlock(
//           context
//             .polkadotJs()
//             .tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, 1)
//             .signAsync(ethan)
//         )
//       );
//     });

//     it({
//       id: "",
//       title: "should succeed",
//       test: async () => {
//         const candidateState = await context
//           .polkadotJs()
//           .query.parachainStaking.candidateInfo(ethan.address);
//         expect(candidateState.unwrap().toJSON()).to.deep.equal({
//           bond: numberToHex(MIN_GLMR_STAKING),
//           delegationCount: 0,
//           totalCounted: numberToHex(MIN_GLMR_STAKING),
//           lowestTopDelegationAmount: 0,
//           highestBottomDelegationAmount: 0,
//           lowestBottomDelegationAmount: 0,
//           topCapacity: "Empty",
//           bottomCapacity: "Empty",
//           request: null,
//           status: { active: null },
//         });
//       },
//     });
//   },
// });

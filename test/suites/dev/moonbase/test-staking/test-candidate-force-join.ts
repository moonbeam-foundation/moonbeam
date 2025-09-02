import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { MIN_GLMR_STAKING, alith, ethan, faith } from "@moonwall/util";

describeSuite({
  id: "D023401",
  title: "Staking - Candidate Force Join - bond less than min",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
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
    });

    it({
      id: "T01",
      title: "should succeed",
      test: async () => {
        const minCandidateStk = psConst.minCandidateStk;
        const block = await createBlock(
          sudo(psTx.forceJoinCandidates(ethan.address, minCandidateStk.subn(10), 1)).signAsync(
            alith
          )
        );
        expect(block.result!.successful).to.be.true;
      },
    });

    it({
      id: "T02",
      title: "should fail",
      test: async () => {
        const block = await createBlock(
          sudo(psTx.forceJoinCandidates(ethan.address, MIN_GLMR_STAKING, 1)).signAsync(alith),
          {
            allowFailures: true,
            expectEvents: [context.polkadotJs().events.sudo.Sudid],
          }
        );

        const { events } = block.result!;
        const event = events.find((event) => {
          const module = event.event.data[0].toPrimitive().err?.module;
          const parachainStaking = 12;
          const candidateExists = "0x06000000";
          return module?.index === parachainStaking && module?.error === candidateExists;
        });
        expect(event).not.to.be.undefined;
      },
    });

    it({
      id: "T03",
      title: "should fail",
      test: async () => {
        const block = await createBlock(
          sudo(psTx.forceJoinCandidates(faith.address, MIN_GLMR_STAKING, 0)).signAsync(alith)
        );
        const { events } = block.result!;
        const event = events.find((event) => {
          const module = event.event.data[0].toPrimitive().err?.module;
          const parachainStaking = 12;
          const tooLowCandidateCountWeightHintJoinCandidates = "0x1c000000";
          return (
            module?.index === parachainStaking &&
            module?.error === tooLowCandidateCountWeightHintJoinCandidates
          );
        });
        expect(event).not.to.be.undefined;
      },
    });
  },
});

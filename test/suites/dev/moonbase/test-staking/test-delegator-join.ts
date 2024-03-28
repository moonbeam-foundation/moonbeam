import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { MIN_GLMR_DELEGATOR, alith, baltathar, ethan } from "@moonwall/util";

describeSuite({
  id: "D013443",
  title: "Staking - Delegator Join",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "bond less than min should fail",
      test: async () => {
        const minDelegatorStk = context.polkadotJs().consts.parachainStaking.minDelegation;
        const block = await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.delegate(alith.address, minDelegatorStk.subn(10), 0, 0)
            .signAsync(ethan)
        );
        expect(block.result!.successful).to.be.false;
        expect(block.result!.error!.name).to.equal("DelegationBelowMin");
      },
    });

    it({
      id: "T02",
      title: "candidate not exists should fail",
      test: async () => {
        const block = await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.delegate(baltathar.address, MIN_GLMR_DELEGATOR, 0, 0)
            .signAsync(ethan)
        );
        expect(block.result!.successful!).to.be.false;
        expect(block.result!.error!.name).to.equal("CandidateDNE");
      },
    });

    it({
      id: "T03",
      title: "candidate not exists and self should fail",
      test: async () => {
        const block = await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.delegate(ethan.address, MIN_GLMR_DELEGATOR, 0, 0)
            .signAsync(ethan)
        );
        expect(block.result!.successful!).to.be.false;
        expect(block.result!.error!.name).to.equal("CandidateDNE");
      },
    });

    it({
      id: "T04",
      title: "already a candidate should fail",
      test: async () => {
        const block = await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.delegate(alith.address, MIN_GLMR_DELEGATOR, 0, 0)
            .signAsync(alith)
        );
        expect(block.result!.successful!).to.be.false;
        expect(block.result!.error!.name).to.equal("CandidateExists");
      },
    });
  },
});

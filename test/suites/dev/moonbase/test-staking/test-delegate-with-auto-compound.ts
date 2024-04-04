import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { MIN_GLMR_DELEGATOR, alith, baltathar, ethan, goliath } from "@moonwall/util";

describeSuite({
  id: "D013413",
  title: "Staking - Delegate With Auto-Compound",
  foundationMethods: "dev",
  testCases: ({ it, log, context }) => {
    it({
      id: "T01",
      title: " bond less than min should fail",
      test: async () => {
        const minDelegatorStk = context.polkadotJs().consts.parachainStaking.minDelegation;
        const block = await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.delegateWithAutoCompound(
              alith.address,
              minDelegatorStk.subn(10),
              50,
              0,
              0,
              0
            )
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
            .tx.parachainStaking.delegateWithAutoCompound(
              baltathar.address,
              MIN_GLMR_DELEGATOR,
              50,
              0,
              0,
              0
            )
            .signAsync(ethan)
        );
        expect(block.result!.successful).to.be.false;
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
            .tx.parachainStaking.delegateWithAutoCompound(
              ethan.address,
              MIN_GLMR_DELEGATOR,
              50,
              0,
              0,
              0
            )
            .signAsync(ethan)
        );
        expect(block.result!.successful).to.be.false;
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
            .tx.parachainStaking.delegateWithAutoCompound(
              alith.address,
              MIN_GLMR_DELEGATOR,
              50,
              0,
              0,
              0
            )
            .signAsync(alith)
        );
        expect(block.result!.successful).to.be.false;
        expect(block.result!.error!.name).to.equal("CandidateExists");
      },
    });

    it({
      id: "T05",
      title: "101% should fail",
      test: async () => {
        expect(
          async () =>
            await context.createBlock(
              context
                .polkadotJs()
                .tx.parachainStaking.delegateWithAutoCompound(
                  alith.address,
                  MIN_GLMR_DELEGATOR,
                  101,
                  0,
                  0,
                  0
                )
                .signAsync(goliath)
            )
        ).rejects.toThrowError("1002: Verification Error: Runtime error: Execution failed");
      },
    });
  },
});

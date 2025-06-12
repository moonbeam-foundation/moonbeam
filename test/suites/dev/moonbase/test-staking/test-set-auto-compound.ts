import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import {
  MIN_GLMR_DELEGATOR,
  MIN_GLMR_STAKING,
  alith,
  baltathar,
  dorothy,
  ethan,
  faith,
} from "@moonwall/util";

describeSuite({
  id: "D023467",
  title: "Staking - Set Auto-Compound",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      await context.createBlock(
        [
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
            .signAsync(ethan),
        ],
        { allowFailures: false }
      );
    });

    it({
      id: "T01",
      title: "delegation not exists should fail",
      test: async () => {
        const { result } = await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.setAutoCompound(baltathar.address, 50, 0, 1)
            .signAsync(ethan)
        );
        expect(result!.successful).to.be.false;
        expect(result!.error!.name).to.equal("DelegationDNE");
      },
    });

    it({
      id: "T02",
      title: "delegator not exists should fail",
      test: async () => {
        const { result } = await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.setAutoCompound(alith.address, 50, 0, 0)
            .signAsync(dorothy)
        );
        expect(result!.successful).to.be.false;
        expect(result!.error!.name).to.equal("DelegatorDNE");
      },
    });

    it({
      id: "T03",
      title: "wrong delegation hint should fail",
      test: async () => {
        await context.createBlock(
          [
            context
              .polkadotJs()
              .tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, 1)
              .signAsync(baltathar),
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
              .signAsync(faith),
          ],
          { allowFailures: false }
        );

        const block = await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.setAutoCompound(alith.address, 50, 0, 0)
            .signAsync(faith)
        );
        expect(block.result!.successful).to.be.false;
        expect(block.result!.error!.name).to.equal("TooLowDelegationCountToAutoCompound");
      },
    });
  },
});

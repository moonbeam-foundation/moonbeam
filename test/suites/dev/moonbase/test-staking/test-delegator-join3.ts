import "@moonbeam-network/api-augment";
import {
  MIN_GLMR_DELEGATOR,
  MIN_GLMR_STAKING,
  alith,
  baltathar,
  beforeAll,
  charleth,
  describeSuite,
  ethan,
  expect,
} from "moonwall";
import { verifyDelegatorStateMatchesFreezes } from "../../../../helpers/staking-freezes";

describeSuite({
  id: "D023346",
  title: "Staking - Delegator Join - wrong candidate delegation hint",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    beforeAll(async () => {
      await context.createBlock([
        context
          .polkadotJs()
          .tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, 1)
          .signAsync(baltathar),
        context
          .polkadotJs()
          .tx.parachainStaking.delegateWithAutoCompound(
            alith.address,
            MIN_GLMR_DELEGATOR,
            0,
            0,
            0,
            0
          )
          .signAsync(charleth),
        context
          .polkadotJs()
          .tx.parachainStaking.delegateWithAutoCompound(
            baltathar.address,
            MIN_GLMR_DELEGATOR,
            0,
            0,
            0,
            0
          )
          .signAsync(ethan),
      ]);

      // Verify delegator states match freezes after initial delegations
      await verifyDelegatorStateMatchesFreezes(charleth.address as `0x${string}`, context);
      await verifyDelegatorStateMatchesFreezes(ethan.address as `0x${string}`, context);
    });

    it({
      id: "T01",
      title: "should fail",
      test: async () => {
        const block = await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.delegateWithAutoCompound(
              alith.address,
              MIN_GLMR_DELEGATOR,
              0,
              0,
              0,
              1
            )
            .signAsync(ethan)
        );
        expect(block.result!.successful!).to.be.false;
        expect(block.result!.error!.name).to.equal("TooLowCandidateDelegationCountToDelegate");
      },
    });
  },
});

import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, GLMR, generateKeyringPair } from "@moonwall/util";
import { stringToU8a } from "@polkadot/util";
import {
  RELAYCHAIN_ARBITRARY_ADDRESS_1,
  VESTING_PERIOD,
  calculate_vested_amount,
  getAccountPayable,
} from "../../../../helpers";

describeSuite({
  id: "D020707",
  title: "Crowdloan",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    const relayAccount = generateKeyringPair("ed25519");
    const toAssociateAccount = generateKeyringPair();

    it({
      id: "T01",
      title: "should be able to associate identity",
      test: async function () {
        await context.createBlock(
          context.polkadotJs().tx.sudo.sudo(
            context.polkadotJs().tx.crowdloanRewards.initializeRewardVec([
              [RELAYCHAIN_ARBITRARY_ADDRESS_1, ALITH_ADDRESS, 1_500_000n * GLMR],
              [relayAccount.addressRaw, null, 1_500_000n * GLMR],
            ])
          )
        );

        const initBlock = await context.polkadotJs().query.crowdloanRewards.initRelayBlock();

        // Complete initialization
        await context.createBlock(
          context
            .polkadotJs()
            .tx.sudo.sudo(
              context
                .polkadotJs()
                .tx.crowdloanRewards.completeInitialization(initBlock.toBigInt() + VESTING_PERIOD)
            )
        );

        const isInitialized = await context.polkadotJs().query.crowdloanRewards.initialized();

        expect(isInitialized.isTrue).to.be.true;

        // relayAccount should be in the unassociated contributions
        expect(
          (
            await context
              .polkadotJs()
              .query.crowdloanRewards.unassociatedContributions(relayAccount.addressRaw as string)
          )
            .unwrap()
            .totalReward.toBigInt()
        ).to.equal(1_500_000n * GLMR);

        // toAssociateAccount should not be in accounts payable
        expect(await getAccountPayable(context, toAssociateAccount.address)).to.be.null;

        const message = new Uint8Array([
          ...stringToU8a("<Bytes>"),
          ...stringToU8a("moonbase-"),
          ...toAssociateAccount.addressRaw,
          ...stringToU8a("</Bytes>"),
        ]);
        // Construct the signature
        const signature = { Ed25519: relayAccount.sign(message) };

        // Associate the identity
        await context.createBlock(
          context
            .polkadotJs()
            .tx.crowdloanRewards.associateNativeIdentity(
              toAssociateAccount.address,
              relayAccount.addressRaw,
              signature
            )
        );

        // relayAccount should no longer be in the unassociated contributions
        expect(
          (
            await context
              .polkadotJs()
              .query.crowdloanRewards.unassociatedContributions(relayAccount.addressRaw)
          ).isEmpty
        ).to.be.true;

        // toAssociateAccount should now be in accounts payable
        const rewardInfo = await getAccountPayable(context, toAssociateAccount.address);

        expect(rewardInfo!.totalReward.toBigInt()).to.equal(1_500_000n * GLMR);
        expect(rewardInfo!.claimedReward.toBigInt()).to.equal(450_000n * GLMR);

        // three blocks elapsed
        const claimed = await calculate_vested_amount(
          rewardInfo!.totalReward.toBigInt(),
          rewardInfo!.claimedReward.toBigInt(),
          3n
        );

        await context.createBlock(
          context.polkadotJs().tx.crowdloanRewards.claim().signAsync(toAssociateAccount)
        );

        // Claimed amount should match
        expect(
          (await getAccountPayable(context, toAssociateAccount.address))!.claimedReward.toBigInt()
        ).to.equal(claimed);
      },
    });
  },
});

import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { GLMR, generateKeyringPair } from "@moonwall/util";
import { stringToU8a } from "@polkadot/util";
import { VESTING_PERIOD, getAccountPayable } from "../../../../helpers";

describeSuite({
  id: "D020713",
  title: "Crowdloan - Update Address Relay",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    const relayAccount = generateKeyringPair("ed25519");
    const relayAccount2 = generateKeyringPair("ed25519");
    const firstAccount = generateKeyringPair();
    const toAssociateAccount = generateKeyringPair();

    it({
      id: "T01",
      title: "should be able to change reward address with relay keys",
      test: async function () {
        await context.createBlock(
          context.polkadotJs().tx.sudo.sudo(
            context.polkadotJs().tx.crowdloanRewards.initializeRewardVec([
              [relayAccount.addressRaw, firstAccount.address, 1_500_000n * GLMR],
              [relayAccount2.addressRaw, firstAccount.address, 1_500_000n * GLMR],
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

        // toAssociateAccount should not be in accounts payable
        expect(await getAccountPayable(context, toAssociateAccount.address)).to.be.null;

        const message = new Uint8Array([
          ...stringToU8a("<Bytes>"),
          ...stringToU8a("moonbase-"),
          ...toAssociateAccount.addressRaw,
          ...firstAccount.addressRaw,
          ...stringToU8a("</Bytes>"),
        ]);

        // Construct the signatures
        const signature1 = { Ed25519: relayAccount.sign(message) };
        const signature2 = { Ed25519: relayAccount2.sign(message) };

        const proofs = [
          [relayAccount.addressRaw, signature1],
          [relayAccount2.addressRaw, signature2],
        ] as any[];
        // Associate the identity
        await context.createBlock(
          context
            .polkadotJs()
            .tx.crowdloanRewards.changeAssociationWithRelayKeys(
              toAssociateAccount.address,
              firstAccount.address,
              proofs
            )
        );

        // toAssociateAccount should now be in accounts payable
        const rewardInfo = await getAccountPayable(context, toAssociateAccount.address);

        expect(rewardInfo!.totalReward.toBigInt()).to.equal(3_000_000n * GLMR);
      },
    });
  },
});

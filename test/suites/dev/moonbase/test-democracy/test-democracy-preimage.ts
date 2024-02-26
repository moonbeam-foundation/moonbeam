import "@moonbeam-network/api-augment";
import { describeSuite, expect, notePreimage } from "@moonwall/cli";
import { ALITH_ADDRESS, MICROGLMR, alith } from "@moonwall/util";
import { blake2AsHex } from "@polkadot/util-crypto";
import { generatePrivateKey, privateKeyToAccount } from "viem/accounts";

describeSuite({
  id: "D010901",
  title: "Democracy - Preimage",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    it({
      id: "T01",
      title: "should be notable",
      test: async function () {
        const encodedProposal =
          context
            .polkadotJs()
            .tx.parachainStaking.setParachainBondAccount(
              privateKeyToAccount(generatePrivateKey()).address
            )
            .method.toHex() || "";
        const encodedHash = blake2AsHex(encodedProposal);
        await context.createBlock(context.polkadotJs().tx.preimage.notePreimage(encodedProposal));

        const preimageStatus = (
          await context.polkadotJs().query.preimage.requestStatusFor(encodedHash)
        ).toHuman();
        expect(preimageStatus).to.not.be.undefined;

        // TODO: uncomment when we have types
        //expect(preimageStatus.unwrap().isUnrequested).to.be.true;

        // TODO: change syntax when we have types
        const proposer = preimageStatus!["Unrequested"]["ticket"][0];
        const balance = preimageStatus!["Unrequested"]["ticket"][1].replaceAll(/,/g, "");
        expect(proposer.toLowerCase()).to.eq(ALITH_ADDRESS.toLowerCase());
        expect(BigInt(balance)).to.eq(5002200n * MICROGLMR);
      },
    });

    it({
      id: "T02",
      title: "should be forgettable immediatly",
      test: async function () {
        const encodedHash = await notePreimage(
          context,
          context
            .polkadotJs()
            .tx.parachainStaking.setParachainBondAccount(
              privateKeyToAccount(generatePrivateKey()).address
            ),
          alith
        );

        await context.createBlock(context.polkadotJs().tx.preimage.unnotePreimage(encodedHash));

        const preimageStatus = (await context
          .polkadotJs()
          .query.preimage.statusFor(encodedHash)) as any;
        expect(preimageStatus.isSome).to.be.false;
      },
    });
  },
});

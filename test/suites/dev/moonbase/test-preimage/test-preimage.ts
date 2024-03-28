import "@moonbeam-network/api-augment";
import { describeSuite, expect, notePreimage } from "@moonwall/cli";
import { ALITH_ADDRESS, MICROGLMR, alith } from "@moonwall/util";
import { blake2AsHex } from "@polkadot/util-crypto";
import { generatePrivateKey, privateKeyToAccount } from "viem/accounts";

describeSuite({
  id: "D012901",
  title: "Preimage - general",
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

        const preimageStatus = await context
          .polkadotJs()
          .query.preimage.requestStatusFor(encodedHash);
        expect(preimageStatus.isEmpty).to.not.be.true;
        expect(preimageStatus.unwrap().isUnrequested).to.be.true;

        const proposer = preimageStatus.unwrap().asUnrequested.ticket[0].toString();
        const balance = preimageStatus.unwrap().asUnrequested.ticket[1].toBigInt();
        expect(proposer.toLowerCase()).to.eq(ALITH_ADDRESS.toLowerCase());
        expect(balance).to.eq(5002200n * MICROGLMR);
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

import "@moonbeam-network/api-augment";
import { describeSuite, expect, beforeEach, notePreimage } from "@moonwall/cli";
import { ALITH_ADDRESS, GLMR, PROPOSAL_AMOUNT, alith } from "@moonwall/util";
import { generatePrivateKey, privateKeyToAccount } from "viem/accounts";

describeSuite({
  id: "D0806",
  title: "Democracy - proposing a vote",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let encodedHash: string;
    let randomAddress: string;

    beforeEach(async () => {
      randomAddress = privateKeyToAccount(generatePrivateKey()).address;
      const proposal = context
        .polkadotJs()
        .tx.parachainStaking.setParachainBondAccount(randomAddress);
      encodedHash = await notePreimage(context, proposal, alith);
      await context.createBlock(
        context.polkadotJs().tx.democracy.propose(
          {
            Lookup: {
              hash: encodedHash,
              len: proposal.method.encodedLength,
            },
          },
          PROPOSAL_AMOUNT
        )
      );
    });

    it({
      id: "T01",
      title: "should not create a referendum",
      test: async function () {
        const referendumCount = await context.polkadotJs().query.democracy.referendumCount();
        expect(referendumCount.toBigInt()).to.equal(0n);
      },
    });

    it({
      id: "T02",
      title: "should increase the number of proposals to 2",
      test: async function () {
        const publicPropCount = await context.polkadotJs().query.democracy.publicPropCount();
        expect(publicPropCount.toBigInt()).to.equal(2n);
      },
    });

    it({
      id: "T03",
      title: "should create a proposal",
      test: async function () {
        const publicProps = await context.polkadotJs().query.democracy.publicProps();

        expect(publicProps[publicProps.length - 1][1].asLookup.hash_.toHex().toString()).to.equal(
          encodedHash
        );
        expect(publicProps[publicProps.length - 1][2].toString()).toBe(ALITH_ADDRESS);
      },
    });

    it({
      id: "T04",
      title: "should include a deposit of 1000 TOKENs",
      test: async function () {
        const depositOf = await context.polkadotJs().query.democracy.depositOf(0);
        expect(depositOf.unwrap()[1].toBigInt()).to.equal(1000n * GLMR);
      },
    });
  },
});

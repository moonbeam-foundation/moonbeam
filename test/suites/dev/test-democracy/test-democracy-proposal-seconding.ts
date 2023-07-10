import "@moonbeam-network/api-augment";
import { describeSuite, expect, beforeEach, notePreimage, beforeAll } from "@moonwall/cli";
import { ALITH_ADDRESS, GLMR, PROPOSAL_AMOUNT, alith } from "@moonwall/util";
import { generatePrivateKey, privateKeyToAccount } from "viem/accounts";

describeSuite({
  id: "D0805",
  title: "Democracy - Seconding a proposal",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let encodedHash: string;
    let launchPeriod: bigint;

    beforeAll(async () => {
      const proposal = context
        .polkadotJs()
        .tx.parachainStaking.setParachainBondAccount(ALITH_ADDRESS);
      launchPeriod = context.polkadotJs().consts.democracy.launchPeriod.toBigInt();
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
      await context.createBlock(context.polkadotJs().tx.democracy.second(0));
    });

    it({
      id: "T01",
      title: "should succeed",
      test: async function () {
        const publicProps = await context.polkadotJs().query.democracy.publicProps();
        expect(publicProps[0][1].asLookup.hash_.toHex().toString()).to.equal(encodedHash);
        expect(publicProps[0][2].toString()).to.equal(ALITH_ADDRESS);

        const depositOf = await context.polkadotJs().query.democracy.depositOf(0);
        expect(depositOf.unwrap()[1].toBigInt()).to.equal(1000n * GLMR);
        expect(depositOf.unwrap()[0][1].toString()).to.equal(ALITH_ADDRESS);
      },
    });

    it({
      id: "T02",
      title: "should have a launch period of 7200",
      test: async function () {
        expect(launchPeriod).to.equal(7200n);
      },
    });

    it({
      id: "T03",
      title: "should end-up in a valid referendum",
      timeout: 1000000,
      test: async function () {
        // let Launchperiod elapse to turn the proposal into a referendum
        // launchPeriod minus the 3 blocks that already elapsed
        for (let i = 0; i < Number(launchPeriod) - 3; i++) {
          await context.createBlock();
        }
        const referendumCount = await context.polkadotJs().query.democracy.referendumCount();
        expect(referendumCount.toBigInt()).to.equal(1n);

        const publicPropCount = await context.polkadotJs().query.democracy.publicPropCount();
        expect(publicPropCount.toBigInt()).to.equal(1n);

        const referendumInfoOf = await context.polkadotJs().query.democracy.referendumInfoOf(0);
        expect(referendumInfoOf.unwrap().asOngoing.proposal.asLookup.hash_.toHex()).to.equal(
          encodedHash
        );
      },
    });
  },
});

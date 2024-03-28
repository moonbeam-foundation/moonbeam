import "@moonbeam-network/api-augment";
import {
  beforeEach,
  describeSuite,
  expect,
  fastFowardToNextEvent,
  maximizeConvictionVotingOf,
  whiteListTrackNoSend,
} from "@moonwall/cli";
import { ALITH_ADDRESS, GLMR, KeyringPair, ethan, generateKeyringPair } from "@moonwall/util";

describeSuite({
  id: "D013303",
  title: "Referenda - Submit",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let whitelistedHash: string;
    let currentRef: number;
    let randomAddress: string;
    let randomAccount: KeyringPair;
    let randBlocksPerRound: number;

    beforeEach(async () => {
      randBlocksPerRound = Math.floor(Math.random() * 1000) + 200;
      randomAccount = generateKeyringPair();
      randomAddress = randomAccount.address;

      const proposal = context.pjsApi.tx.parachainStaking.setParachainBondAccount(randomAddress);

      const { whitelistedHash: wlHash } = await whiteListTrackNoSend(context, proposal);
      whitelistedHash = wlHash;

      currentRef = (await context.polkadotJs().query.referenda.referendumCount()).toNumber() - 1;

      if (currentRef < 0) {
        throw new Error("No referenda found");
      }

      await context.createBlock(context.pjsApi.tx.referenda.placeDecisionDeposit(currentRef));
      await context.createBlock(
        context.pjsApi.tx.sudo.sudo(
          context.pjsApi.tx.balances.forceSetBalance(ethan.address, 1_000_000_000n * GLMR)
        )
      );
    });

    it({
      id: "T01",
      title: "should succeed with enough votes",
      test: async () => {
        await maximizeConvictionVotingOf(context, [ethan], currentRef);
        await context.createBlock();

        const referendumInfoOf = (
          await context.polkadotJs().query.referenda.referendumInfoFor(currentRef)
        ).unwrap();
        const onGoing = referendumInfoOf.asOngoing;

        expect(
          onGoing.proposal.asLookup.hash_.toHex(),
          "Current proposal Hash doesn't match expected"
        ).to.equal(whitelistedHash);
        expect(onGoing.tally.ayes.toBigInt() > 1000n * GLMR, "Unexpected voting amounts").to.be
          .true;
        expect(onGoing.tally.support.toBigInt() > 1000n * GLMR, "Unexpected voting amounts").to.be
          .true;

        await fastFowardToNextEvent(context); // Fast forward past preparation
        await fastFowardToNextEvent(context); // Fast forward past decision
        await fastFowardToNextEvent(context); // Fast forward past enactment

        const finishedReferendum = (
          await context.polkadotJs().query.referenda.referendumInfoFor(currentRef)
        ).unwrap();

        expect(finishedReferendum.isApproved, "Not approved").to.be.true;
        expect(finishedReferendum.isOngoing, "Still ongoing").to.be.false;
        expect(finishedReferendum.isTimedOut, "Timed out").to.be.false;

        const parachainBondInfo = await context.pjsApi.query.parachainStaking.parachainBondInfo();
        expect(parachainBondInfo.account.toString()).toBe(randomAddress);
      },
    });

    it({
      id: "T02",
      title: "should fail with enough no votes",
      modifier: "skip",
      test: async () => {
        await context.createBlock([
          context
            .polkadotJs()
            .tx.convictionVoting.vote(currentRef, {
              Standard: {
                vote: { aye: false, conviction: "Locked6x" },
                balance: 10n * GLMR,
              },
            })
            .signAsync(ethan),
        ]);

        const referendumInfoOf = (
          await context.polkadotJs().query.referenda.referendumInfoFor(currentRef)
        ).unwrap();
        const onGoing = referendumInfoOf.asOngoing;

        expect(
          onGoing.proposal.asLookup.hash_.toHex(),
          "Current proposal hash doesn't match expected"
        ).to.equal(whitelistedHash);
        expect(onGoing.tally.nays.toBigInt(), "Incorrect Nay votes").to.equal(60n * GLMR);
        expect(onGoing.tally.support.toBigInt() === 0n, "Incorrect support count").to.be.true;
        await fastFowardToNextEvent(context); // Fast forward past preparation

        // TODO: Renable when we have a way of reaching the end of this referendum
        //        without fastfowarding 200k blocks

        // await fastFowardToNextEvent(context); // Fast forward past decision

        // const finishedReferendum = (
        //   await context.polkadotJs().query.referenda.referendumInfoFor(currentRef)
        // ).unwrap();

        // expect(finishedReferendum.isApproved).to.be.false;
        // expect(finishedReferendum.isRejected).to.be.true;
        // log(finishedReferendum.asRejected.toHuman());

        // const parachainBondInfo = await context
        //   .polkadotJs()
        //   .query.parachainStaking.parachainBondInfo();
        // expect(parachainBondInfo.account.toString()).not.toBe(randomAddress);
      },
    });

    it({
      id: "T03",
      title: "should be votable while staked",
      test: async () => {
        await context.createBlock(
          context.polkadotJs().tx.balances.transferAllowDeath(randomAccount.address, 100n * GLMR)
        );

        await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.delegate(ALITH_ADDRESS, 90n * GLMR, 0, 0)
            .signAsync(randomAccount)
        );
        currentRef = (await context.polkadotJs().query.referenda.referendumCount()).toNumber() - 1;
        const { result } = await context.createBlock(
          context
            .polkadotJs()
            .tx.convictionVoting.vote(currentRef, {
              Standard: {
                vote: { aye: false, conviction: "Locked1x" },
                balance: 90n * GLMR,
              },
            })
            .signAsync(randomAccount)
        );
        expect(result?.successful).to.be.true;

        const locks = await context.polkadotJs().query.balances.locks(randomAccount.address);
        expect(locks.length).to.be.equal(2, "Failed to incur two locks");
        expect(locks[0].amount.toBigInt()).to.be.equal(90n * GLMR);
        expect(locks[0].id.toHuman()).to.be.equal("stkngdel");
        expect(locks[1].amount.toBigInt()).to.be.equal(90n * GLMR);
        expect(locks[1].id.toHuman()).to.be.equal("pyconvot");
      },
    });
  },
});

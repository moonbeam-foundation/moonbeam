import "@moonbeam-network/api-augment";
import { beforeEach, describeSuite, expect, instantFastTrack } from "@moonwall/cli";
import { ALITH_ADDRESS, GLMR, KeyringPair, VOTE_AMOUNT, generateKeyringPair } from "@moonwall/util";

describeSuite({
  id: "D010902",
  title: "Democracy - Referendum",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let encodedHash: string;
    let currentRef: bigint;
    let randomAddress: string;
    let randomAccount: KeyringPair;

    beforeEach(async () => {
      log(`Disabled test D010904 (Gov V1)`);
      return;

      randomAccount = generateKeyringPair();
      randomAddress = randomAccount.address;
      encodedHash = await instantFastTrack(
        context,
        context.polkadotJs().tx.parachainStaking.setParachainBondAccount(randomAddress)
      );
      currentRef = (await context.polkadotJs().query.democracy.referendumCount()).toBigInt() - 1n;
    });

    it({
      id: "T01",
      title: "should succeed with enough votes",
      test: async function () {
        log(`Disabled test D010904 (Gov V1)`);
        return;
        await context.createBlock(
          context.polkadotJs().tx.democracy.vote(currentRef, {
            Standard: { balance: VOTE_AMOUNT, vote: { aye: true, conviction: 1 } },
          })
        );

        const referendumInfoOf = (
          await context.polkadotJs().query.democracy.referendumInfoOf(currentRef)
        ).unwrap();
        const onGoing = referendumInfoOf.asOngoing;

        expect(onGoing.proposal.asLookup.hash_.toHex()).to.equal(encodedHash);
        expect(onGoing.tally.ayes.toBigInt()).to.equal(10n * GLMR);
        expect(onGoing.tally.turnout.toBigInt()).to.equal(10n * GLMR);

        const blockNumber = (await context.polkadotJs().rpc.chain.getHeader()).number.toNumber();
        for (let i = 0; i < onGoing.end.toNumber() - blockNumber + 1; i++) {
          await context.createBlock();
        }

        const finishedReferendum = (
          await context.polkadotJs().query.democracy.referendumInfoOf(currentRef)
        ).unwrap();

        expect(finishedReferendum.isFinished).to.be.true;
        expect(finishedReferendum.asFinished.approved.isTrue).to.be.true;

        const parachainBondInfo = await context
          .polkadotJs()
          .query.parachainStaking.parachainBondInfo();
        expect(parachainBondInfo.account.toString()).to.equal(randomAddress);
      },
    });

    it({
      id: "T02",
      title: "should fail with enough no votes",
      test: async function () {
        log(`Disabled test D010904 (Gov V1)`);
        return;
        await context.createBlock(
          context.polkadotJs().tx.democracy.vote(currentRef, {
            Standard: { balance: VOTE_AMOUNT, vote: { aye: false, conviction: 1 } },
          })
        );

        const referendumInfoOf = (
          await context.polkadotJs().query.democracy.referendumInfoOf(currentRef)
        ).unwrap();
        const onGoing = referendumInfoOf.asOngoing;

        expect(onGoing.proposal.asLookup.hash_.toHex()).to.equal(encodedHash);
        expect(onGoing.tally.nays.toBigInt()).to.equal(10n * GLMR);
        expect(onGoing.tally.turnout.toBigInt()).to.equal(10n * GLMR);

        const blockNumber = (await context.polkadotJs().rpc.chain.getHeader()).number.toNumber();
        for (let i = 0; i < onGoing.end.toNumber() - blockNumber + 1; i++) {
          await context.createBlock();
        }
        const finishedReferendum = (
          await context.polkadotJs().query.democracy.referendumInfoOf(currentRef)
        ).unwrap();

        expect(finishedReferendum.isFinished).to.be.true;
        expect(finishedReferendum.asFinished.approved.isFalse).to.be.true;

        const parachainBondInfo = await context
          .polkadotJs()
          .query.parachainStaking.parachainBondInfo();
        expect(parachainBondInfo.account.toString()).not.toBe(randomAddress);
      },
    });

    it({
      id: "T03",
      title: "should be votable while staked",
      test: async function () {
        log(`Disabled test D010904 (Gov V1)`);
        return;
        encodedHash = await instantFastTrack(
          context,
          context.polkadotJs().tx.system.remark("Just a simple vote"),
          { votingPeriod: 10, delayPeriod: 1 }
        );
        await context.createBlock(
          context.polkadotJs().tx.balances.transferAllowDeath(randomAccount.address, 100n * GLMR)
        );
        await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.delegate(ALITH_ADDRESS, 90n * GLMR, 0, 0)
            .signAsync(randomAccount)
        );
        currentRef = (await context.polkadotJs().query.democracy.referendumCount()).toBigInt() - 1n;
        const { result } = await context.createBlock(
          context
            .polkadotJs()
            .tx.democracy.vote(currentRef, {
              Standard: { balance: 90n * GLMR, vote: { aye: false, conviction: 1 } },
            })
            .signAsync(randomAccount)
        );

        expect(result!.successful).to.be.true;

        const locks = await context.polkadotJs().query.balances.locks(randomAccount.address);
        expect(locks.length).to.be.equal(2, "Failed to incur two locks");
        expect(locks[0].amount.toBigInt()).to.be.equal(90n * GLMR);
        expect(locks[0].id.toHuman()).to.be.equal("stkngdel");
        expect(locks[1].amount.toBigInt()).to.be.equal(90n * GLMR);
        expect(locks[1].id.toHuman()).to.be.equal("democrac");
      },
    });
  },
});

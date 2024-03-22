import "@moonbeam-network/api-augment";
import {
  beforeEach,
  describeSuite,
  execOpenTechCommitteeProposal,
  expect,
  fastFowardToNextEvent,
  maximizeConvictionVotingOf,
  notePreimage,
} from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  GLMR,
  KeyringPair,
  charleth,
  dorothy,
  ethan,
  faith,
  generateKeyringPair,
} from "@moonwall/util";

describeSuite({
  id: "D013303",
  title: "Referenda - Submit",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let wLPreimage: string;
    let preimageLen: bigint;
    let currentRef: number;
    let randomAddress: string;
    let randomAccount: KeyringPair;
    let randBlocksPerRound: number;

    beforeEach(async () => {
      randBlocksPerRound = Math.floor(Math.random() * 1000) + 200;

      const preimageHash = await notePreimage(
        context,
        // context.polkadotJs().tx.parachainStaking.setParachainBondAccount(randomAddress)
        context.pjsApi.tx.parachainStaking.setBlocksPerRound(randBlocksPerRound)
      );
      const preimageQuery = await context.pjsApi.query.preimage.requestStatusFor(preimageHash);
      preimageLen = preimageQuery.unwrap().asUnrequested.len.toBigInt();

      const dispatchWLCall = context.pjsApi.tx.whitelist.dispatchWhitelistedCall(
        preimageHash,
        preimageLen,
        {
          refTime: 2_000_000_000,
          proofSize: 100_000,
        }
      );

      wLPreimage = await notePreimage(context, dispatchWLCall);
      const wLPreimageLen = dispatchWLCall.encodedLength - 2;

      console.log(
        `📝 DispatchWhitelistedCall preimage noted: ${wLPreimage.slice(0, 6)}...${wLPreimage.slice(
          -4
        )}, len: ${wLPreimageLen}`
      );

      randomAccount = generateKeyringPair();
      randomAddress = randomAccount.address;

      const openGovProposal = await context.pjsApi.tx.referenda
        .submit(
          {
            Origins: { whitelistedcaller: "WhitelistedCaller" },
          },
          { Lookup: { hash: wLPreimage, len: wLPreimageLen } },
          { After: { After: 0 } }
        )
        .signAsync(faith);
      const { result } = await context.createBlock(openGovProposal);

      const whitelistCall = context.pjsApi.tx.whitelist.whitelistCall(preimageHash);
      await execOpenTechCommitteeProposal(context, whitelistCall);

      currentRef = (await context.polkadotJs().query.referenda.referendumCount()).toNumber() - 1;

      if (currentRef < 0) {
        throw new Error("No referenda found");
      }

      await context.createBlock(context.pjsApi.tx.referenda.placeDecisionDeposit(currentRef));
      await context.createBlock(
        context.pjsApi.tx.sudo.sudo(
          context.pjsApi.tx.balances.forceSetBalance(charleth.address, 1_000_000_000n * GLMR)
        )
      );
      await context.createBlock(
        context.pjsApi.tx.sudo.sudo(
          context.pjsApi.tx.balances.forceSetBalance(dorothy.address, 1_000_000_000n * GLMR)
        )
      );
      await context.createBlock(
        context.pjsApi.tx.sudo.sudo(
          context.pjsApi.tx.balances.forceSetBalance(ethan.address, 1_000_000_000n * GLMR)
        )
      );
    });

    it({
      id: "T01",
      title: "should succeed with enough votes",
      test: async function () {
        await maximizeConvictionVotingOf(context, [ethan], currentRef);
        await context.createBlock();

        const referendumInfoOf = (
          await context.polkadotJs().query.referenda.referendumInfoFor(currentRef)
        ).unwrap();
        const onGoing = referendumInfoOf.asOngoing;

        expect(
          onGoing.proposal.asLookup.hash_.toHex(),
          "Current proposal Hash doesn't match expected"
        ).to.equal(wLPreimage);
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

        // await fastFowardToNextEvent(context);

        const roundInfo = await context.pjsApi.query.parachainStaking.round();
        expect(roundInfo.length.toNumber(), "Storage unchanged").to.equal(randBlocksPerRound);
      },
    });

    it({
      id: "T02",
      title: "should fail with enough no votes",
      test: async function () {
        await context.createBlock([
          context
            .polkadotJs()
            .tx.convictionVoting.vote(currentRef, {
              Standard: {
                vote: { aye: false, conviction: "Locked6x" },
                balance: 10n * GLMR,
              },
            })
            .signAsync(charleth),
        ]);

        const referendumInfoOf = (
          await context.polkadotJs().query.referenda.referendumInfoFor(currentRef)
        ).unwrap();
        const onGoing = referendumInfoOf.asOngoing;

        expect(
          onGoing.proposal.asLookup.hash_.toHex(),
          "Current proposal hash doesn't match expected"
        ).to.equal(wLPreimage);
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
      test: async function () {
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

        expect(result!.successful).to.be.true;

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

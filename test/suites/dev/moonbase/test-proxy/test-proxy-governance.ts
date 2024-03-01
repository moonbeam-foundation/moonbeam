import "@moonbeam-network/api-augment";
import {
  beforeAll,
  describeSuite,
  execCouncilProposal,
  execTechnicalCommitteeProposal,
  expect,
} from "@moonwall/cli";
import { GLMR, VOTE_AMOUNT, dorothy, ethan } from "@moonwall/util";

const proposalHash = "0xf3d039875302d49d52fb1af6877a2c46bc55b004afb8130f94dd9d0489ca3185";

const disableGovTest = true;

describeSuite({
  id: "D013004",
  title: "Proxing governance",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      log(`Disabled test D012804 (Gov V1)`);
      return;

      await execCouncilProposal(
        context,
        context.polkadotJs().tx.democracy.externalProposeMajority({
          Lookup: {
            hash: proposalHash,
            // this test does not test scheduling, therefore this lenght should not
            // matter
            len: 22,
          },
        })
      );
      await execTechnicalCommitteeProposal(
        context,
        context.polkadotJs().tx.democracy.fastTrack(proposalHash, 5, 0)
      );
    });

    it({
      id: "T01",
      title: "should be able to vote on behalf of the delegate account",
      test: async function () {
        log(`Disabled test D012804 (Gov V1)`);
        return;
        const referendumCount = await context.polkadotJs().query.democracy.referendumCount();
        expect(referendumCount.toBigInt(), "Test expects only a single referendum").to.equal(1n);

        await context.createBlock(
          context.polkadotJs().tx.proxy.addProxy(ethan.address, "Governance", 0).signAsync(dorothy)
        );

        const dorothyPreBalance = (
          await context.polkadotJs().query.system.account(dorothy.address)
        ).data.free.toBigInt();

        const expectEvents = [
          context.polkadotJs().events.proxy.ProxyExecuted,
          context.polkadotJs().events.democracy.Voted,
        ];

        const { result } = await context.createBlock(
          context
            .polkadotJs()
            .tx.proxy.proxy(
              dorothy.address,
              "Governance",
              context.polkadotJs().tx.democracy.vote(0, {
                Standard: { balance: VOTE_AMOUNT, vote: { aye: true, conviction: 1 } },
              })
            )
            .signAsync(ethan),
          { expectEvents, allowFailures: false }
        );

        expect(
          result!.events.find((event) =>
            context.polkadotJs().events.proxy.ProxyExecuted.is(event.event)
          )!.event.data.result.isOk
        ).toBe(true);

        // Verify that dorothy hasn't paid for the transaction but the vote locked her tokens
        const dorothyAccountData = await context.polkadotJs().query.system.account(dorothy.address);
        expect(dorothyAccountData.data.free.toBigInt()).to.equal(dorothyPreBalance);
        expect(dorothyAccountData.data.frozen.toBigInt()).to.equal(VOTE_AMOUNT);

        const referendumInfoOf = (
          await context.polkadotJs().query.democracy.referendumInfoOf(0)
        ).unwrap();
        const onGoing = referendumInfoOf.asOngoing;

        expect(onGoing.proposal.asLookup.hash_.toHex(), "Vote is not registerd").to.equal(
          proposalHash
        );
        expect(onGoing.tally.ayes.toBigInt()).to.equal(10n * GLMR);
        expect(onGoing.tally.turnout.toBigInt()).to.equal(10n * GLMR);
      },
    });
  },
});

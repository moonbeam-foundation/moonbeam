import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { GLMR, VOTE_AMOUNT, dorothy, ethan } from "@moonwall/util";

const proposalHash = "0xf3d039875302d49d52fb1af6877a2c46bc55b004afb8130f94dd9d0489ca3185";

describeSuite({
  id: "D010404",
  title: "Proxing governance",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      await context.createBlock(
        context.polkadotJs().tx.referenda.submit(
          {
            Origins: { generalAdmin: "GeneralAdmin" },
          },
          { Lookup: { hash: proposalHash, len: 22 } },
          { After: { After: 0 } }
        )
      );
    });

    it({
      id: "T01",
      title: "should be able to vote on behalf of the delegate account",
      test: async function () {
        const referendumCount = await context.polkadotJs().query.referenda.referendumCount();
        expect(referendumCount.toBigInt(), "Test expects only a single referendum").to.equal(1n);

        await context.createBlock(
          context.polkadotJs().tx.proxy.addProxy(ethan.address, "Governance", 0).signAsync(dorothy)
        );

        const dorothyPreBalance = (
          await context.polkadotJs().query.system.account(dorothy.address)
        ).data.free.toBigInt();

        const expectEvents = [context.polkadotJs().events.proxy.ProxyExecuted];

        const { result } = await context.createBlock(
          context
            .polkadotJs()
            .tx.proxy.proxy(
              dorothy.address,
              "Governance",
              context.polkadotJs().tx.convictionVoting.vote(0, {
                Standard: {
                  vote: { aye: true, conviction: "Locked1x" },
                  balance: VOTE_AMOUNT,
                },
              })
            )
            .signAsync(ethan),
          { expectEvents, allowFailures: false }
        );

        expect(
          result?.events.find((event) =>
            context.polkadotJs().events.proxy.ProxyExecuted.is(event.event)
          )?.event.data.result.isOk
        ).toBe(true);

        // Verify that dorothy hasn't paid for the transaction but the vote locked her tokens
        const dorothyAccountData = await context.polkadotJs().query.system.account(dorothy.address);
        expect(dorothyAccountData.data.free.toBigInt()).to.equal(dorothyPreBalance);
        expect(dorothyAccountData.data.frozen.toBigInt()).to.equal(VOTE_AMOUNT);

        const referendumInfoOf = (
          await context.polkadotJs().query.referenda.referendumInfoFor(0)
        ).unwrap();
        const onGoing = referendumInfoOf.asOngoing;

        expect(onGoing.proposal.asLookup.hash_.toHex(), "Vote is not registerd").to.equal(
          proposalHash
        );
        expect(onGoing.tally.ayes.toBigInt()).to.equal(10n * GLMR);
        expect(onGoing.tally.support.toBigInt()).to.equal(10n * GLMR);
      },
    });
  },
});

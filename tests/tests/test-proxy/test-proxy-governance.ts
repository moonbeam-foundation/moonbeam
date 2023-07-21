import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { dorothy, ethan } from "../../util/accounts";
import { GLMR, VOTE_AMOUNT } from "../../util/constants";
import { execCouncilProposal, execTechnicalCommitteeProposal } from "../../util/governance";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";

const proposalHash = "0xf3d039875302d49d52fb1af6877a2c46bc55b004afb8130f94dd9d0489ca3185";

describeDevMoonbeam("Proxing governance", (context) => {
  before("Create accounts and fast-tracking referundum", async () => {
    await execCouncilProposal(
      context,
      context.polkadotApi.tx.democracy.externalProposeMajority({
        Lookup: {
          hash: proposalHash,
          // this test does not test scheduling, therefore this lenght should not
          // matter
          len: 22,
        },
      } as any)
    );
    await execTechnicalCommitteeProposal(
      context,
      context.polkadotApi.tx.democracy.fastTrack(proposalHash, 5, 0)
    );
  });

  it("should be able to vote on behalf of the delegate account", async function () {
    // Verify that one referundum is triggered
    let referendumCount = await context.polkadotApi.query.democracy.referendumCount();
    expect(referendumCount.toBigInt()).to.equal(1n);

    // Dorothy add proxy right to ethan for governance only
    await context.createBlock(
      context.polkadotApi.tx.proxy.addProxy(ethan.address, "Governance", 0).signAsync(dorothy)
    );

    // Ethan vote as Dorothy
    const dorothyPreBalance = (
      await context.polkadotApi.query.system.account(dorothy.address)
    ).data.free.toBigInt();

    const {
      result: { events },
    } = await context.createBlock(
      context.polkadotApi.tx.proxy
        .proxy(
          dorothy.address,
          "Governance",
          context.polkadotApi.tx.democracy.vote(0, {
            Standard: { balance: VOTE_AMOUNT, vote: { aye: true, conviction: 1 } },
          })
        )
        .signAsync(ethan)
    );

    // Check events
    expect(context.polkadotApi.events.balances.Locked.is(events[2].event)).to.be.true;
    expect(context.polkadotApi.events.proxy.ProxyExecuted.is(events[3].event)).to.be.true;
    expect(context.polkadotApi.events.democracy.Voted.is(events[1].event)).to.be.true;
    expect(events[3].event.data[0].toString()).to.equal("Ok");
    expect(context.polkadotApi.events.treasury.Deposit.is(events[5].event)).to.be.true;
    expect(context.polkadotApi.events.system.ExtrinsicSuccess.is(events[7].event)).to.be.true;

    // Verify that dorothy hasn't paid for the transaction but the vote locked her tokens
    let dorothyAccountData = await context.polkadotApi.query.system.account(dorothy.address);
    expect(dorothyAccountData.data["free"].toBigInt()).to.equal(dorothyPreBalance);
    expect(dorothyAccountData.data["frozen"].toBigInt()).to.equal(VOTE_AMOUNT);

    // Verify that vote is registered
    const referendumInfoOf = (
      await context.polkadotApi.query.democracy.referendumInfoOf(0)
    ).unwrap() as any;
    const onGoing = referendumInfoOf.asOngoing;

    expect(onGoing.proposal.asLookup.hash_.toHex()).to.equal(proposalHash);
    expect(onGoing.tally.ayes.toBigInt()).to.equal(10n * GLMR);
    expect(onGoing.tally.turnout.toBigInt()).to.equal(10n * GLMR);
  });
});

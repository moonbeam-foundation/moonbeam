import "@moonbeam-network/api-augment";

import { expect } from "chai";
import { alith, baltathar, charleth, dorothy } from "../../util/accounts";

import { proposeReferendaAndDeposit, maximizeConvictionVotingOf } from "../../util/governance";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";

describeDevMoonbeam("Referenda - GeneralAdmin", (context) => {
  let refIndex: Number;
  let proposalHash: String;
  before("Prepare pre-image and proposal and 3 members TC", async () => {
    // Just build the arguments. They dont matter that much though, since
    // we will not make sure it executes in the relay
    const transactWeights = context.polkadotApi.createType("PalletXcmTransactorTransactWeights", {
      transactRequiredWeightAtMost: 10000,
      overallWeight: 10000,
    });

    let fee = context.polkadotApi.createType("PalletXcmTransactorCurrencyPayment", {
      currency: {
        AsMultiLocation: {
          V1: {
            parents: 1,
            interior: {
              Here: null,
            },
          },
        },
      },
      feeAmount: 10000,
    }) as any;

    // The proposal itself
    const proposal = (context.polkadotApi.tx.xcmTransactor as any).hrmpManage(
      { Accept: 2000 },
      fee,
      transactWeights
    ) as any;

    // The origin we want to use, post a referenda and deposit.
    [refIndex, proposalHash] = await proposeReferendaAndDeposit(context, alith, proposal, {
      Origins: "GeneralAdmin",
    });
  });

  it("should succeed to call hrmpManage through generalAdmin", async function () {
    this.timeout(500000);

    // Vote with everything they have with those accounts
    await maximizeConvictionVotingOf(context, [alith, baltathar, charleth], refIndex);
    let refInfo = (await context.polkadotApi.query.referenda.referendumInfoFor(refIndex)) as any;

    let blockNumber = (await context.polkadotApi.rpc.chain.getHeader()).number.toNumber();
    // Pass decision phase
    // We need to advance until the next alarm
    for (
      let i = 0;
      i < refInfo.unwrap().asOngoing.alarm.unwrap()[0].toNumber() - blockNumber + 1;
      i++
    ) {
      await context.createBlock();
    }

    blockNumber = (await context.polkadotApi.rpc.chain.getHeader()).number.toNumber();
    refInfo = (await context.polkadotApi.query.referenda.referendumInfoFor(refIndex)) as any;

    // Pass confirmation phase
    // We need to advance until the next alarm
    for (
      let i = 0;
      i < refInfo.unwrap().asOngoing.alarm.unwrap()[0].toNumber() - blockNumber + 1;
      i++
    ) {
      await context.createBlock();
    }

    const track = refInfo.unwrap().asOngoing.track;
    const tracks = await context.polkadotApi.consts.referenda.tracks;
    const minEnactmentPeriod = tracks.find(([index, info]) => index.toString() == track)[1]
      .minEnactmentPeriod;

    blockNumber = (await context.polkadotApi.rpc.chain.getHeader()).number.toNumber();

    // Proposal should have been approved, we need to know at which point the agenda was filled
    // We should expect the proposal to be scheduled after minEnactmentPeriod
    let scheduledAgenda = await context.polkadotApi.query.scheduler.agenda(
      blockNumber + minEnactmentPeriod.toNumber() - 1
    );
    expect(scheduledAgenda[0].unwrap().call.asLookup.hash_.toHex().toString()).to.be.eq(
      proposalHash
    );

    // Run until the enactment block
    for (let i = 0; i < minEnactmentPeriod.toNumber() - 1; i++) {
      await context.createBlock();
    }

    // Filter for HrmpManagementSent events
    const records = (await context.polkadotApi.query.system.events()) as any;
    const events = records.filter(
      ({ event }) => event.section == "xcmTransactor" && event.method == "HrmpManagementSent"
    );

    // It executed!
    expect(events).to.have.lengthOf(1);
  });
});

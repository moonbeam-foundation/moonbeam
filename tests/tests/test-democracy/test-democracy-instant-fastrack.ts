import "@moonbeam-network/api-augment";

import { Result } from "@polkadot/types";
import { SpRuntimeDispatchError } from "@polkadot/types/lookup";
import { expect } from "chai";

import { alith, baltathar, charleth } from "../../util/accounts";
import {
  execCouncilProposal,
  execTechnicalCommitteeProposal,
  notePreimage,
} from "../../util/governance";
import { describeDevMoonbeam, DevTestContext } from "../../util/setup-dev-tests";

const setupProposalAnd3TechnicalCommittee = async (context: DevTestContext) => {
  let proposalHash = await notePreimage(
    context,
    context.polkadotApi.tx.parachainStaking.setParachainBondAccount(alith.address),
    alith
  );
  await execCouncilProposal(
    context,
    context.polkadotApi.tx.democracy.externalProposeMajority(proposalHash)
  );

  await context.createBlock(
    context.polkadotApi.tx.sudo.sudo(
      context.polkadotApi.tx.techCommitteeCollective.setMembers(
        [alith.address, baltathar.address, charleth.address],
        alith.address,
        (
          await context.polkadotApi.query.techCommitteeCollective.members()
        ).length
      )
    )
  );
  return proposalHash;
};

describeDevMoonbeam("Democracy - Instant FastTracking", (context) => {
  let proposalHash: string;

  before("Prepare pre-image and proposal and 3 members TC", async () => {
    proposalHash = await setupProposalAnd3TechnicalCommittee(context);
  });

  it("should fail with less than 2/3rd of the council", async function () {
    const { events } = await execTechnicalCommitteeProposal(
      context,
      context.polkadotApi.tx.democracy.fastTrack(proposalHash, 1, 1),
      [alith],
      1
    );

    // Verify it passed
    expect(
      events.findIndex(
        ({ event: { section, method } }) =>
          section == "techCommitteeCollective" && method == "Executed"
      ),
      "Technical Committee Wrong event"
    ).to.equal(1);
    expect((events[1].event.data[1] as Result<any, SpRuntimeDispatchError>).asErr.isBadOrigin).to.be
      .true;

    // Verify the proposal is NOT sent to referendum
    expect((await context.polkadotApi.query.democracy.referendumCount()).toNumber()).to.be.equal(0);
  });
});

describeDevMoonbeam("Democracy - Instant FastTracking", (context) => {
  let proposalHash: string;

  before("Prepare pre-image and proposal and 3 members TC", async () => {
    proposalHash = await setupProposalAnd3TechnicalCommittee(context);
  });

  it("should succeed with than 2/3rd of the council", async function () {
    const { events } = await execTechnicalCommitteeProposal(
      context,
      context.polkadotApi.tx.democracy.fastTrack(proposalHash, 1, 1),
      [alith, baltathar],
      2
    );

    expect(
      events.findIndex(
        ({ event: { section, method } }) =>
          section == "techCommitteeCollective" && method == "Executed"
      ),
      "Technical Committee Wrong event"
    ).to.equal(4);
    expect((events[4].event.data[1] as Result<any, SpRuntimeDispatchError>).isOk).to.be.true;

    // Verify the proposal is NOT sent to referendum
    expect((await context.polkadotApi.query.democracy.referendumCount()).toNumber()).to.be.equal(1);
  });
});

describeDevMoonbeam("Democracy - Instant FastTracking", (context) => {
  let proposalHash: string;

  before("Prepare pre-image and proposal and 3 members TC", async () => {
    proposalHash = await setupProposalAnd3TechnicalCommittee(context);
  });

  it("should succeed with the full council", async function () {
    const { events } = await execTechnicalCommitteeProposal(
      context,
      context.polkadotApi.tx.democracy.fastTrack(proposalHash, 1, 1),
      [alith, baltathar, charleth],
      3
    );

    expect(
      events.findIndex(
        ({ event: { section, method } }) =>
          section == "techCommitteeCollective" && method == "Executed"
      ),
      "Technical Committee Wrong event"
    ).to.equal(4);
    expect((events[4].event.data[1] as Result<any, SpRuntimeDispatchError>).isOk).to.be.true;

    // Verify the proposal is NOT sent to referendum
    expect((await context.polkadotApi.query.democracy.referendumCount()).toNumber()).to.be.equal(1);
  });
});

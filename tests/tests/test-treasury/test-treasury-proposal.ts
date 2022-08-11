import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { alith, baltathar, charleth, dorothy, ethan } from "../../util/accounts";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";

describeDevMoonbeam("Treasury proposal #1", (context) => {
  it("should not be able to be approved by a non-council member", async function () {
    // Ethan submit a treasury proposal

    await context.createBlock(
      context.polkadotApi.tx.treasury.proposeSpend(10, baltathar.address).signAsync(ethan)
    );

    // Verify that the proposal is submitted
    let proposalCount = await context.polkadotApi.query.treasury.proposalCount();
    expect(proposalCount.toHuman() === "1").to.equal(true, "new proposal should have been added");

    // Try to approve the proposal directly (must be fail)
    let approvals = (await context.polkadotApi.query.treasury.approvals()) as any;
    expect(approvals.length).to.equal(0, "No proposal must have been approved");
  });
});

describeDevMoonbeam("Treasury proposal #2", (context) => {
  it("should not be able to be rejected by a non-council member", async function () {
    // Ethan submit a treasury proposal

    await context.createBlock(
      context.polkadotApi.tx.treasury.proposeSpend(10, baltathar.address).signAsync(ethan)
    );

    // Verify that the proposal is submitted
    let proposalCount = await context.polkadotApi.query.treasury.proposalCount();
    expect(proposalCount.toHuman() === "1").to.equal(true, "new proposal should have been added");

    // Try to reject the proposal directly (must be fail)
    await context.polkadotApi.tx.treasury.rejectProposal(0).signAsync(ethan);
    expect(await context.polkadotApi.query.treasury.proposals(0)).not.equal(
      null,
      "The proposal should not have been deleted"
    );
  });
});

describeDevMoonbeam("Treasury proposal #3", (context) => {
  // prettier-ignore
  it(
    "should be rejected if three-fifths of the treasury council did not vote in favor",
    async function () {
      // Ethan submit a treasury proposal

      await context.createBlock(
        context.polkadotApi.tx.treasury.proposeSpend(10, baltathar.address).signAsync(ethan)
      );

      // Verify that the proposal is submitted
      let proposalCount = await context.polkadotApi.query.treasury.proposalCount();
      expect(proposalCount.toHuman() === "1").to.equal(true, "new proposal should have been added");

      // A council member attempts to approve the proposal on behalf of the council
      // (must fail because there is not a quorum)
      await context.createBlock(
        context.polkadotApi.tx.treasuryCouncilCollective
          .propose(1, context.polkadotApi.tx.treasury.approveProposal(0), 1_000)
          .signAsync(charleth)
      );

      // Verify that the proposal is not deleted
      expect(await context.polkadotApi.query.treasury.proposals(0)).not.equal(
        null,
        "The proposal must not have been deleted"
      );
    }
  );
});

describeDevMoonbeam("Treasury proposal #4", (context) => {
  // prettier-ignore
  it(
    "should not be rejected by less than half of the members of the treasury council",
    async function () {
      // Ethan submit a treasury proposal

      await context.createBlock(
        context.polkadotApi.tx.treasury.proposeSpend(10, baltathar.address).signAsync(ethan)
      );

      // Verify that the proposal is submitted
      let proposalCount = await context.polkadotApi.query.treasury.proposalCount();
      expect(proposalCount.toHuman() === "1").to.equal(true, "new proposal should have been added");

      // A council member attempts to reject the proposal on behalf of the council
      // (must fail because there is not a quorum)
      await context.createBlock(
        context.polkadotApi.tx.treasuryCouncilCollective
          .propose(1, context.polkadotApi.tx.treasury.rejectProposal(0), 1_000)
          .signAsync(charleth)
      );

      // Verify that the proposal is not approved
      let approvals = (await context.polkadotApi.query.treasury.approvals()) as any;
      expect(approvals.length).to.equal(0, "No proposal should have been approved");
    }
  );
});

describeDevMoonbeam("Treasury proposal #5", (context) => {
  it("should be approvable by root", async function () {
    // Ethan submit a treasury proposal

    await context.createBlock(
      context.polkadotApi.tx.treasury.proposeSpend(10, baltathar.address).signAsync(ethan)
    );

    // Verify that the proposal is submitted
    let proposalCount = await context.polkadotApi.query.treasury.proposalCount();
    expect(proposalCount.toHuman() === "1").to.equal(true, "new proposal should have been added");

    // Root approve the proposal directly
    await context.createBlock(
      context.polkadotApi.tx.sudo
        .sudo(context.polkadotApi.tx.treasury.approveProposal(0))
        .signAsync(alith)
    );

    // Verify that the proposal is approved
    let approvals = (await context.polkadotApi.query.treasury.approvals()) as any;
    expect(approvals.length).to.equal(1, "One proposal should have been approved");
  });
});

describeDevMoonbeam("Treasury proposal #6", (context) => {
  it("should be rejectable by root", async function () {
    // Ethan submit a treasury proposal

    await context.createBlock(
      context.polkadotApi.tx.treasury.proposeSpend(10, baltathar.address).signAsync(ethan)
    );

    // Verify that the proposal is submitted
    let proposalCount = await context.polkadotApi.query.treasury.proposalCount();
    expect(proposalCount.toHuman() === "1").to.equal(true, "new proposal should have been added");

    // Root approve the proposal directly
    await context.createBlock(
      context.polkadotApi.tx.sudo
        .sudo(context.polkadotApi.tx.treasury.rejectProposal(0))
        .signAsync(alith)
    );

    // Verify that the proposal is deleted
    expect(await (await context.polkadotApi.query.treasury.proposals(0)).toHuman()).to.equal(
      null,
      "The proposal must have been deleted"
    );
  });
});

describeDevMoonbeam("Treasury proposal #7", (context) => {
  // prettier-ignore
  it(
    "should NO LONGER be approved if the three fifths of the council voted for it",
    async function () {
      // To run this long scenarios, we have to go through a lot of steps, so anyway we won't be
      // able to keep this tests within 5 seconds.
      this.timeout(10_000);

      // Ethan submit a treasury proposal

      await context.createBlock(
        context.polkadotApi.tx.treasury.proposeSpend(10, baltathar.address).signAsync(ethan)
      );

      // Verify that the proposal is submitted
      let proposalCount = (await context.polkadotApi.query.treasury.proposalCount()) as any;
      expect(proposalCount.toBigInt()).to.equal(1n, "new proposal should have been added");

      // Charleth submit the proposal to the council (and therefore implicitly votes for)
      const {
        result: { events: proposalEvents },
      } = await context.createBlock(
        context.polkadotApi.tx.councilCollective
          .propose(2, context.polkadotApi.tx.treasury.approveProposal(0), 1_000)
          .signAsync(charleth)
      );
      const proposalHash = proposalEvents
        .find(({ event: { method } }) => method.toString() == "Proposed")
        .event.data[2].toHex() as string;

      // Charleth & Dorothy vote for this proposal and close it
      await context.createBlock([
        context.polkadotApi.tx.councilCollective.vote(proposalHash, 0, true).signAsync(charleth),
        context.polkadotApi.tx.councilCollective
          .vote(proposalHash, 0, true)
          .signAsync(dorothy, { nonce: 0 }),
        context.polkadotApi.tx.councilCollective
          .close(proposalHash, 0, 800_000_000, 1_000)
          .signAsync(dorothy, { nonce: 1 }),
      ]);

      // Verify that the proposal is not approved
      let approvals = (await context.polkadotApi.query.treasury.approvals()) as any;
      expect(approvals.length).to.equal(0, "No proposal should have been approved");
    }
  );
});

describeDevMoonbeam("Treasury proposal #8", (context) => {
  // prettier-ignore
  it(
    "should NO LONGER be rejected if the half of the council voted against it",
    async function () {
      // Ethan submit a treasury proposal

      await context.createBlock(
        context.polkadotApi.tx.treasury.proposeSpend(10, baltathar.address).signAsync(ethan)
      );

      // Verify that the proposal is submitted
      let proposalCount = (await context.polkadotApi.query.treasury.proposalCount()) as any;
      expect(proposalCount.toBigInt()).to.equal(1n, "new proposal should have been added");

      // Charleth proposed that the council reject the treasury proposal
      // (and therefore implicitly votes for)
      const {
        result: { events: rejectEvents },
      } = await context.createBlock(
        context.polkadotApi.tx.councilCollective
          .propose(2, context.polkadotApi.tx.treasury.rejectProposal(0), 1_000)
          .signAsync(charleth)
      );
      const councilProposalHash = rejectEvents
        .find(({ event: { method } }) => method.toString() == "Proposed")
        .event.data[2].toHex() as string;

      // Charleth & Dorothy vote for against proposal and close it
      await context.createBlock([
        context.polkadotApi.tx.councilCollective
          .vote(councilProposalHash, 0, true)
          .signAsync(charleth),
        context.polkadotApi.tx.councilCollective
          .vote(councilProposalHash, 0, true)
          .signAsync(dorothy),
      ]);
      const {
        result: { events: closeEvents },
      } = await context.createBlock(
        context.polkadotApi.tx.councilCollective
          .close(councilProposalHash, 0, 800_000_000, 1_000)
          .signAsync(dorothy)
      );

      // Verify that the proposal is not deleted
      expect(await context.polkadotApi.query.treasury.proposals(0)).not.equal(
        null,
        "The proposal must not have been deleted"
      );
    }
  );
});

describeDevMoonbeam("Treasury proposal #9", (context) => {
  // prettier-ignore
  it(
    "should be approved if the three fifths of the treasury council voted for it",
    async function () {
      // To run this long scenarios, we have to go through a lot of steps, so anyway we won't be
      // able to keep this tests within 5 seconds.
      this.timeout(10_000);

      // Ethan submit a treasury proposal

      await context.createBlock(
        context.polkadotApi.tx.treasury.proposeSpend(10, baltathar.address).signAsync(ethan)
      );

      // Verify that the proposal is submitted
      let proposalCount = (await context.polkadotApi.query.treasury.proposalCount()) as any;
      expect(proposalCount.toBigInt()).to.equal(1n, "new proposal should have been added");

      // Charleth submit the proposal to the council (and therefore implicitly votes for)
      const {
        result: { events: proposalEvents },
      } = await context.createBlock(
        context.polkadotApi.tx.treasuryCouncilCollective
          .propose(2, context.polkadotApi.tx.treasury.approveProposal(0), 1_000)
          .signAsync(charleth)
      );
      const proposalHash = proposalEvents
        .find(({ event: { method } }) => method.toString() == "Proposed")
        .event.data[2].toHex() as string;

      // Charleth & Dorothy vote for this proposal and close it
      await context.createBlock([
        context.polkadotApi.tx.treasuryCouncilCollective
          .vote(proposalHash, 0, true)
          .signAsync(charleth),
        context.polkadotApi.tx.treasuryCouncilCollective
          .vote(proposalHash, 0, true)
          .signAsync(dorothy, { nonce: 0 }),
        context.polkadotApi.tx.treasuryCouncilCollective
          .close(proposalHash, 0, 800_000_000, 1_000)
          .signAsync(dorothy, { nonce: 1 }),
      ]);

      // Verify that the proposal is approved
      let approvals = (await context.polkadotApi.query.treasury.approvals()) as any;
      expect(approvals.length).to.equal(1, "one proposal should have been approved");
    }
  );
});

describeDevMoonbeam("Treasury proposal #10", (context) => {
  it("should be rejected if the half of the treasury council voted against it", async function () {
    // Ethan submit a treasury proposal

    await context.createBlock(
      context.polkadotApi.tx.treasury.proposeSpend(10, baltathar.address).signAsync(ethan)
    );

    // Verify that the proposal is submitted
    let proposalCount = (await context.polkadotApi.query.treasury.proposalCount()) as any;
    expect(proposalCount.toBigInt()).to.equal(1n, "new proposal should have been added");

    // Charleth proposed that the council reject the treasury proposal
    // (and therefore implicitly votes for)
    const {
      result: { events: rejectEvents },
    } = await context.createBlock(
      context.polkadotApi.tx.treasuryCouncilCollective
        .propose(2, context.polkadotApi.tx.treasury.rejectProposal(0), 1_000)
        .signAsync(charleth)
    );
    const councilProposalHash = rejectEvents
      .find(({ event: { method } }) => method.toString() == "Proposed")
      .event.data[2].toHex() as string;

    // Charleth & Dorothy vote for against proposal and close it
    await context.createBlock([
      context.polkadotApi.tx.treasuryCouncilCollective
        .vote(councilProposalHash, 0, true)
        .signAsync(charleth),
      context.polkadotApi.tx.treasuryCouncilCollective
        .vote(councilProposalHash, 0, true)
        .signAsync(dorothy),
    ]);
    const {
      result: { events: closeEvents },
    } = await context.createBlock(
      context.polkadotApi.tx.treasuryCouncilCollective
        .close(councilProposalHash, 0, 800_000_000, 1_000)
        .signAsync(dorothy)
    );
    // method: 'Rejected', section: 'treasury', index: '0x1103',
    expect(closeEvents.map(({ event }) => event.index.toHuman())).to.contain("0x1103");

    // Verify that the proposal is deleted
    expect((await context.polkadotApi.query.treasury.proposals(0)).toHuman()).to.equal(
      null,
      "The proposal must have been deleted"
    );
  });
});

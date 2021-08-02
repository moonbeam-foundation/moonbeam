import { expect } from "chai";
import Keyring from "@polkadot/keyring";
import {
  ALITH_PRIV_KEY,
  BALTATHAR,
  CHARLETH_PRIV_KEY,
  DOROTHY_PRIV_KEY,
  ETHAN_PRIVKEY,
} from "../util/constants";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createBlockWithExtrinsic } from "../util/substrate-rpc";

describeDevMoonbeam("Treasury proposal #1", (context) => {
  it("should not be able to be approved by a non-council member", async function () {
    const keyring = new Keyring({ type: "ethereum" });
    const ethan = await keyring.addFromUri(ETHAN_PRIVKEY, null, "ethereum");

    // Ethan submit a treasurery proposal
    await context.polkadotApi.tx.treasury.proposeSpend(10, BALTATHAR).signAndSend(ethan);
    await context.createBlock();

    // Verify that the proposal is submitted
    let proposalCount = await context.polkadotApi.query.treasury.proposalCount();
    expect(proposalCount.toHuman() === "1").to.equal(true, "new proposal should have been added");

    // Try to approve the proposal directly (must be fail)
    await context.polkadotApi.tx.treasury.approveProposal(0).signAndSend(ethan);
    let approvals = await context.polkadotApi.query.treasury.approvals();
    expect(approvals.length).to.equal(0, "No proposal must have been approved");
  });
});

describeDevMoonbeam("Treasury proposal #2", (context) => {
  it("should not be able to be rejected by a non-council member", async function () {
    const keyring = new Keyring({ type: "ethereum" });
    const ethan = await keyring.addFromUri(ETHAN_PRIVKEY, null, "ethereum");

    // Ethan submit a treasurery proposal
    await context.polkadotApi.tx.treasury.proposeSpend(10, BALTATHAR).signAndSend(ethan);
    await context.createBlock();

    // Verify that the proposal is submitted
    let proposalCount = await context.polkadotApi.query.treasury.proposalCount();
    expect(proposalCount.toHuman() === "1").to.equal(true, "new proposal should have been added");

    // Try to reject the proposal directly (must be fail)
    await context.polkadotApi.tx.treasury.rejectProposal(0).signAndSend(ethan);
    expect(await context.polkadotApi.query.treasury.proposals(0)).not.equal(
      null,
      "The proposal should not have been deleted"
    );
  });
});

describeDevMoonbeam("Treasury proposal #3", (context) => {
  it("should be rejected if three-fifths of the council did not vote in favor", async function () {
    const keyring = new Keyring({ type: "ethereum" });
    const charleth = await keyring.addFromUri(CHARLETH_PRIV_KEY, null, "ethereum");
    const ethan = await keyring.addFromUri(ETHAN_PRIVKEY, null, "ethereum");

    // Ethan submit a treasurery proposal
    await context.polkadotApi.tx.treasury.proposeSpend(10, BALTATHAR).signAndSend(ethan);
    await context.createBlock();

    // Verify that the proposal is submitted
    let proposalCount = await context.polkadotApi.query.treasury.proposalCount();
    expect(proposalCount.toHuman() === "1").to.equal(true, "new proposal should have been added");

    // A council member attempts to approve the proposal on behalf of the council
    // (must fail because there is not a quorum)
    await context.polkadotApi.tx.councilCollective
      .propose(1, context.polkadotApi.tx.treasury.approveProposal(0), 1_000)
      .signAndSend(charleth);
    await context.createBlock();

    // Verify that the proposal is not deleted
    expect(await context.polkadotApi.query.treasury.proposals(0)).not.equal(
      null,
      "The proposal must not have been deleted"
    );
  });
});

describeDevMoonbeam("Treasury proposal #4", (context) => {
  it("should not be rejected by less than half of the members of the Board", async function () {
    const keyring = new Keyring({ type: "ethereum" });
    const charleth = await keyring.addFromUri(CHARLETH_PRIV_KEY, null, "ethereum");
    const ethan = await keyring.addFromUri(ETHAN_PRIVKEY, null, "ethereum");

    // Ethan submit a treasurery proposal
    await context.polkadotApi.tx.treasury.proposeSpend(10, BALTATHAR).signAndSend(ethan);
    await context.createBlock();

    // Verify that the proposal is submitted
    let proposalCount = await context.polkadotApi.query.treasury.proposalCount();
    expect(proposalCount.toHuman() === "1").to.equal(true, "new proposal should have been added");

    // A council member attempts to reject the proposal on behalf of the council
    // (must fail because there is not a quorum)
    await context.polkadotApi.tx.councilCollective
      .propose(1, context.polkadotApi.tx.treasury.rejectProposal(0), 1_000)
      .signAndSend(charleth);
    await context.createBlock();

    // Verify that the proposal is not approved
    let approvals = await context.polkadotApi.query.treasury.approvals();
    expect(approvals.length).to.equal(0, "No proposal should have been approved");
  });
});

describeDevMoonbeam("Treasury proposal #5", (context) => {
  it.skip("should be approvable by root", async function () {
    const keyring = new Keyring({ type: "ethereum" });
    const alith = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
    const ethan = await keyring.addFromUri(ETHAN_PRIVKEY, null, "ethereum");

    // Ethan submit a treasurery proposal
    await context.polkadotApi.tx.treasury.proposeSpend(10, BALTATHAR).signAndSend(ethan);
    await context.createBlock();

    // Verify that the proposal is submitted
    let proposalCount = await context.polkadotApi.query.treasury.proposalCount();
    expect(proposalCount.toHuman() === "1").to.equal(true, "new proposal should have been added");

    // Root approve the proposal directly
    await context.polkadotApi.tx.sudo
      .sudo(context.polkadotApi.tx.treasury.approveProposal(0))
      .signAndSend(alith);
    await context.createBlock();

    // Verify that the proposal is approved
    let approvals = await context.polkadotApi.query.treasury.approvals();
    expect(approvals.length).to.equal(1, "One proposal should have been approved");
  });
});

describeDevMoonbeam("Treasury proposal #6", (context) => {
  it.skip("should be rejectable by root", async function () {
    const keyring = new Keyring({ type: "ethereum" });
    const alith = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
    const ethan = await keyring.addFromUri(ETHAN_PRIVKEY, null, "ethereum");

    // Ethan submit a treasurery proposal
    await context.polkadotApi.tx.treasury.proposeSpend(10, BALTATHAR).signAndSend(ethan);
    await context.createBlock();

    // Verify that the proposal is submitted
    let proposalCount = await context.polkadotApi.query.treasury.proposalCount();
    expect(proposalCount.toHuman() === "1").to.equal(true, "new proposal should have been added");

    // Root approve the proposal directly
    await context.polkadotApi.tx.sudo
      .sudo(context.polkadotApi.tx.treasury.rejectProposal(0))
      .signAndSend(alith);
    await context.createBlock();

    // Verify that the proposal is deleted
    expect(await (await context.polkadotApi.query.treasury.proposals(0)).toHuman()).to.equal(
      null,
      "The proposal must have been deleted"
    );
  });
});

describeDevMoonbeam("Treasury proposal #7", (context) => {
  it("should be approved if the three fifths of the council voted for it", async function () {
    // To run this long scenarios, we have to go through a lot of steps, so anyway we won't be
    // able to keep this tests within 5 seconds.
    this.timeout(10_000);

    const keyring = new Keyring({ type: "ethereum" });
    const charleth = await keyring.addFromUri(CHARLETH_PRIV_KEY, null, "ethereum");
    const dorothy = await keyring.addFromUri(DOROTHY_PRIV_KEY, null, "ethereum");
    const ethan = await keyring.addFromUri(ETHAN_PRIVKEY, null, "ethereum");

    // Ethan submit a treasurery proposal
    await context.polkadotApi.tx.treasury.proposeSpend(10, BALTATHAR).signAndSend(ethan);
    await context.createBlock();

    // Verify that the proposal is submitted
    let proposalCount = await context.polkadotApi.query.treasury.proposalCount();
    expect(proposalCount.toHuman()).to.equal("1", "new proposal should have been added");

    // Charleth submit the proposal to the council (and therefore implicitly votes for)
    const { events: proposalEvents } = await createBlockWithExtrinsic(
      context,
      charleth,
      context.polkadotApi.tx.councilCollective.propose(
        2,
        context.polkadotApi.tx.treasury.approveProposal(0),
        1_000
      )
    );
    const proposalHash = proposalEvents[0].data[2].toHuman();

    // Dorothy vote for this proposal and close it
    await context.polkadotApi.tx.councilCollective.vote(proposalHash, 0, true).signAndSend(dorothy);
    await context.createBlock();
    await context.createBlock();
    await context.polkadotApi.tx.councilCollective
      .close(proposalHash, 0, 800_000_000, 1_000)
      .signAndSend(dorothy);
    await context.createBlock();

    // Verify that the proposal is approved
    let approvals = await context.polkadotApi.query.treasury.approvals();
    console.log(JSON.stringify(approvals));
    expect(approvals.length).to.equal(1, "one proposal should have been approved");
  });
});

describeDevMoonbeam("Treasury proposal #8", (context) => {
  it("should be rejected if the half of the council voted against it", async function () {
    const keyring = new Keyring({ type: "ethereum" });
    const charleth = await keyring.addFromUri(CHARLETH_PRIV_KEY, null, "ethereum");
    const dorothy = await keyring.addFromUri(DOROTHY_PRIV_KEY, null, "ethereum");
    const ethan = await keyring.addFromUri(ETHAN_PRIVKEY, null, "ethereum");

    // Ethan submit a treasurery proposal
    await context.polkadotApi.tx.treasury.proposeSpend(10, BALTATHAR).signAndSend(ethan);
    await context.createBlock();

    // Verify that the proposal is submitted
    let proposalCount = await context.polkadotApi.query.treasury.proposalCount();
    expect(proposalCount.toHuman()).to.equal("1", "new proposal should have been added");

    // Charleth proposed that the council reject the treasury proposal
    // (and therefore implicitly votes for)
    const { events: rejectEvents } = await createBlockWithExtrinsic(
      context,
      charleth,
      context.polkadotApi.tx.councilCollective.propose(
        2,
        context.polkadotApi.tx.treasury.rejectProposal(0),
        1_000
      )
    );
    const councilProposalHash = rejectEvents[0].data[2].toHuman();

    // Dorothy vote for against proposal and close it
    await context.polkadotApi.tx.councilCollective
      .vote(councilProposalHash, 0, true)
      .signAndSend(dorothy);
    await context.createBlock();
    const { events: closeEvents } = await createBlockWithExtrinsic(
      context,
      dorothy,
      context.polkadotApi.tx.councilCollective.close(councilProposalHash, 0, 800_000_000, 1_000)
    );
    // method: 'Rejected', section: 'treasury', index: '0x1103',
    expect(closeEvents.map((e) => e.index.toHuman())).to.contain("0x1103");

    // Verify that the proposal is deleted
    expect((await context.polkadotApi.query.treasury.proposals(0)).toHuman()).to.equal(
      null,
      "The proposal must have been deleted"
    );
  });
});

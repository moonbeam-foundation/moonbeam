import { ApiTypes, SubmittableExtrinsic } from "@polkadot/api/types";
import { KeyringPair } from "@polkadot/keyring/types";
import { blake2AsHex } from "@polkadot/util-crypto";
import { expect } from "chai";

import { alith, baltathar, charleth, dorothy } from "./accounts";
import { DevTestContext } from "./setup-dev-tests";

export const COUNCIL_MEMBERS = [baltathar, charleth, dorothy];
export const COUNCIL_THRESHOLD = Math.ceil((COUNCIL_MEMBERS.length * 2) / 3);
export const TECHNICAL_COMMITTEE_MEMBERS = [alith, baltathar];
export const TECHNICAL_COMMITTEE_THRESHOLD = Math.ceil(
  (TECHNICAL_COMMITTEE_MEMBERS.length * 2) / 3
);

export const notePreimage = async <
  Call extends SubmittableExtrinsic<ApiType>,
  ApiType extends ApiTypes
>(
  context: DevTestContext,
  proposal: Call,
  account: KeyringPair
): Promise<string> => {
  const encodedProposal = proposal.method.toHex() || "";
  await context.createBlock(
    context.polkadotApi.tx.democracy.notePreimage(encodedProposal).signAsync(account)
  );

  return blake2AsHex(encodedProposal);
};

// Creates the Council Proposal and fast track it before executing it
export const instantFastTrack = async (
  context: DevTestContext,
  proposalHash: string,
  { votingPeriod, delayPeriod } = { votingPeriod: 2, delayPeriod: 0 }
) => {
  await execCouncilProposal(
    context,
    context.polkadotApi.tx.democracy.externalProposeMajority(proposalHash)
  );
  await execTechnicalCommitteeProposal(
    context,
    context.polkadotApi.tx.democracy.fastTrack(proposalHash, votingPeriod, delayPeriod)
  );
};

// Creates the Council Proposal
// Vote with the members (all members by default)
// Close it (Execute if successful)
export const execCouncilProposal = async <
  Call extends SubmittableExtrinsic<ApiType>,
  ApiType extends ApiTypes
>(
  context: DevTestContext,
  polkadotCall: Call,
  voters: KeyringPair[] = COUNCIL_MEMBERS,
  threshold: number = COUNCIL_THRESHOLD
) => {
  // Charleth submit the proposal to the council (and therefore implicitly votes for)
  let lengthBound = polkadotCall.encodedLength;
  const { result: proposalResult } = await context.createBlock(
    context.polkadotApi.tx.councilCollective
      .propose(threshold, polkadotCall, lengthBound)
      .signAsync(charleth)
  );

  if (threshold <= 1) {
    // Proposal are automatically executed on threshold <= 1
    return proposalResult;
  }

  expect(proposalResult.successful, `Council proposal refused: ${proposalResult?.error?.name}`).to
    .be.true;
  const proposalHash = proposalResult.events
    .find(({ event: { method } }) => method.toString() == "Proposed")
    .event.data[2].toHex() as string;

  // Dorothy vote for this proposal and close it

  await Promise.all(
    voters.map((voter) =>
      context.polkadotApi.tx.councilCollective.vote(proposalHash, 0, true).signAndSend(voter)
    )
  );
  await context.createBlock();
  return await context.createBlock(
    context.polkadotApi.tx.councilCollective
      .close(proposalHash, 0, 1_000_000_000, lengthBound)
      .signAsync(dorothy)
  );
};

// Creates the Technical Committee Proposal
// Vote with the members (all members by default)
// Close it (Execute if successful)
export const execTechnicalCommitteeProposal = async <
  Call extends SubmittableExtrinsic<ApiType>,
  ApiType extends ApiTypes
>(
  context: DevTestContext,
  polkadotCall: Call,
  voters: KeyringPair[] = TECHNICAL_COMMITTEE_MEMBERS,
  threshold: number = TECHNICAL_COMMITTEE_THRESHOLD
) => {
  // Tech committee members

  // Alith submit the proposal to the council (and therefore implicitly votes for)
  let lengthBound = polkadotCall.encodedLength;
  const { result: proposalResult } = await context.createBlock(
    context.polkadotApi.tx.techCommitteeCollective.propose(threshold, polkadotCall, lengthBound)
  );

  if (threshold <= 1) {
    // Proposal are automatically executed on threshold <= 1
    return proposalResult;
  }

  expect(proposalResult.successful, `Council proposal refused: ${proposalResult?.error?.name}`).to
    .be.true;
  const proposalHash = proposalResult.events
    .find(({ event: { method } }) => method.toString() == "Proposed")
    .event.data[2].toHex() as string;

  // Get proposal count
  const proposalCount = await context.polkadotApi.query.techCommitteeCollective.proposalCount();

  await context.createBlock(
    voters.map((voter) =>
      context.polkadotApi.tx.techCommitteeCollective
        .vote(proposalHash, Number(proposalCount) - 1, true)
        .signAsync(voter)
    )
  );
  const { result: closeResult } = await context.createBlock(
    context.polkadotApi.tx.techCommitteeCollective
      .close(proposalHash, Number(proposalCount) - 1, 1_000_000_000, lengthBound)
      .signAsync(baltathar)
  );
  return closeResult;
};

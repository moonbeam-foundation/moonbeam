import { ApiPromise } from "@polkadot/api";
import { ApiTypes, SubmittableExtrinsic } from "@polkadot/api/types";
import { KeyringPair } from "@polkadot/keyring/types";
import { PalletDemocracyReferendumInfo } from "@polkadot/types/lookup";
import { blake2AsHex } from "@polkadot/util-crypto";
import { expect } from "chai";
import { GLMR } from "../util/constants";

import { alith, baltathar, charleth, dorothy } from "./accounts";
import { DevTestContext } from "./setup-dev-tests";

export const COUNCIL_MEMBERS = [baltathar, charleth, dorothy];
export const COUNCIL_THRESHOLD = Math.ceil((COUNCIL_MEMBERS.length * 2) / 3);
export const TECHNICAL_COMMITTEE_MEMBERS = [alith, baltathar];
export const TECHNICAL_COMMITTEE_THRESHOLD = Math.ceil(
  (TECHNICAL_COMMITTEE_MEMBERS.length * 2) / 3
);

// TODO: Refactor to support both instant sealing and parachain environment
// (using a waitOrCreateNextBlock common function)

export const notePreimage = async <
  Call extends SubmittableExtrinsic<ApiType>,
  ApiType extends ApiTypes
>(
  context: DevTestContext,
  proposal: Call,
  account: KeyringPair = alith
): Promise<string> => {
  const encodedProposal = proposal.method.toHex() || "";
  await context.createBlock(
    context.polkadotApi.tx.preimage.notePreimage(encodedProposal).signAsync(account)
  );

  return blake2AsHex(encodedProposal);
};

// Creates the Council Proposal and fast track it before executing it
export const instantFastTrack = async <
  Call extends SubmittableExtrinsic<ApiType>,
  ApiType extends ApiTypes
>(
  context: DevTestContext,
  proposal: string | Call,
  { votingPeriod, delayPeriod } = { votingPeriod: 2, delayPeriod: 0 }
): Promise<string> => {
  const proposalHash =
    typeof proposal == "string" ? proposal : await notePreimage(context, proposal);

  await execCouncilProposal(
    context,
    context.polkadotApi.tx.democracy.externalProposeMajority({
      Lookup: {
        hash: proposalHash,
        len: typeof proposal == "string" ? proposal : proposal.method.encodedLength,
      },
    } as any)
  );
  await execTechnicalCommitteeProposal(
    context,
    context.polkadotApi.tx.democracy.fastTrack(proposalHash, votingPeriod, delayPeriod)
  );
  return proposalHash;
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
  let lengthBound = polkadotCall.method.encodedLength;
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
      .close(
        proposalHash,
        0,
        {
          refTime: 1_000_000_000,
          proofSize: 0,
        } as any,
        lengthBound
      )
      .signAsync(dorothy)
  );
};

// Proposes referenda and places decision deposit
// Returns referendum index and proposal hash
export const proposeReferendaAndDeposit = async <
  Call extends SubmittableExtrinsic<ApiType>,
  ApiType extends ApiTypes
>(
  context: DevTestContext,
  decisionDepositer: KeyringPair,
  proposal: string | Call,
  origin
): Promise<[Number, String]> => {
  // Fetch proposal hash
  const proposalHash =
    typeof proposal == "string" ? proposal : await notePreimage(context, proposal);

  // Post referenda
  const { result: proposalResult } = await context.createBlock(
    context.polkadotApi.tx.referenda
      .submit(
        origin,
        {
          Lookup: {
            hash: proposalHash,
            len: typeof proposal == "string" ? proposal : proposal.method.encodedLength,
          },
        },
        { At: 0 }
      )
      .signAsync(alith)
  );

  expect(proposalResult.successful, `Unable to post referenda: ${proposalResult?.error?.name}`).to
    .be.true;

  const refIndex = proposalResult.events
    .find(({ event: { method } }) => method.toString() == "Submitted")
    .event.data[0].toString();

  // Place decision deposit
  await context.createBlock(
    context.polkadotApi.tx.referenda.placeDecisionDeposit(refIndex).signAsync(decisionDepositer)
  );

  return [+refIndex, proposalHash];
};

// Proposes referenda and places decision deposit
// Returns referendum index and proposal hash
export const dispatchAsGeneralAdmin = async <
  Call extends SubmittableExtrinsic<ApiType>,
  ApiType extends ApiTypes
>(
  context: DevTestContext,
  call: string | Call
) => {
  // Post referenda
  await context.createBlock(
    context.polkadotApi.tx.sudo.sudo(
      context.polkadotApi.tx.utility.dispatchAs(
        {
          Origins: "GeneralAdmin",
        } as any,
        call
      )
    )
  );
};

// Maximizes conviction voting of some voters
// with respect to an ongoing referenda
// Their whole free balance will be used to vote
export const maximizeConvictionVotingOf = async (
  context: DevTestContext,
  voters: KeyringPair[],
  refIndex: Number
) => {
  // We need to have enough to pay for fee
  const fee = (
    await context.polkadotApi.tx.convictionVoting
      .vote(refIndex as any, {
        Standard: {
          vote: { aye: true, conviction: "Locked6x" },
          balance: await (await context.polkadotApi.query.system.account(alith.address)).data.free,
        },
      })
      .paymentInfo(alith)
  ).partialFee;

  // We vote with everything but fee
  await context.createBlock(
    voters.map(async (voter) =>
      context.polkadotApi.tx.convictionVoting
        .vote(refIndex as any, {
          Standard: {
            vote: { aye: true, conviction: "Locked6x" },
            balance: await (
              await context.polkadotApi.query.system.account(voter.address)
            ).data.free.sub(fee),
          },
        })
        .signAsync(voter)
    )
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
      .close(
        proposalHash,
        Number(proposalCount) - 1,
        {
          refTime: 1_000_000_000,
          proofSize: 64 * 1024,
        } as any,
        lengthBound
      )
      .signAsync(baltathar)
  );
  return closeResult;
};

export const executeProposalWithCouncil = async (api: ApiPromise, encodedHash: string) => {
  let nonce = (await api.rpc.system.accountNextIndex(alith.address)).toNumber();
  let referendumNextIndex = (await api.query.democracy.referendumCount()).toNumber();

  // process.stdout.write(
  //   `Sending council motion (${encodedHash} ` +
  //     `[threashold: 1, expected referendum: ${referendumNextIndex}])...`
  // );
  const callData =
    api.consts.system.version.specVersion.toNumber() >= 2000
      ? { Legacy: encodedHash }
      : encodedHash;

  let external = api.tx.democracy.externalProposeMajority(callData);
  let fastTrack = api.tx.democracy.fastTrack(encodedHash, 1, 0);
  const voteAmount = 1n * 10n ** BigInt(api.registry.chainDecimals[0]);

  process.stdout.write(`Sending motion + fast-track + vote for ${encodedHash}...`);
  await Promise.all([
    api.tx.councilCollective
      .propose(1, external, external.length)
      .signAndSend(alith, { nonce: nonce++ }),
    api.tx.techCommitteeCollective
      .propose(1, fastTrack, fastTrack.length)
      .signAndSend(alith, { nonce: nonce++ }),
    api.tx.democracy
      .vote(referendumNextIndex, {
        Standard: {
          balance: voteAmount,
          vote: { aye: true, conviction: 1 },
        },
      })
      .signAndSend(alith, { nonce: nonce++ }),
  ]);
  process.stdout.write(`✅\n`);

  process.stdout.write(`Waiting for referendum [${referendumNextIndex}] to be executed...`);
  let referenda: PalletDemocracyReferendumInfo = null;
  while (!referenda) {
    referenda = (await api.query.democracy.referendumInfoOf.entries())
      .find(
        (ref) =>
          ref[1].unwrap().isFinished &&
          api.registry.createType("u32", ref[0].toU8a().slice(-4)).toNumber() == referendumNextIndex
      )?.[1]
      .unwrap();
    await new Promise((resolve) => setTimeout(resolve, 1000));
  }
  process.stdout.write(`${referenda.asFinished.approved ? `✅` : `❌`} \n`);
  if (!referenda.asFinished.approved) {
    process.exit(1);
  }
};

export const cancelReferendaWithCouncil = async (api: ApiPromise, refIndex: number) => {
  const proposal = api.tx.democracy.cancelReferendum(refIndex);
  const encodedProposal = proposal.method.toHex();
  const encodedHash = blake2AsHex(encodedProposal);

  let nonce = (await api.rpc.system.accountNextIndex(alith.address)).toNumber();
  await api.tx.democracy.notePreimage(encodedProposal).signAndSend(alith, { nonce: nonce++ });
  await executeProposalWithCouncil(api, encodedHash);
};

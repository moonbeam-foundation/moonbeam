import { type DevModeContext, filterAndApply } from "@moonwall/cli";
import { alith, baltathar } from "@moonwall/util";
import { expectSubstrateEvent } from "./expect.js";
import type { KeyringPair } from "@polkadot/keyring/types";
import type { ApiTypes, SubmittableExtrinsic } from "@polkadot/api/types";

export async function createProposal({
  context,
  track = "root",
  from = alith,
}: {
  context: DevModeContext;
  track?: string;
  from?: KeyringPair;
}) {
  let nonce = (await context.polkadotJs().rpc.system.accountNextIndex(from.address)).toNumber();
  const call = context.polkadotJs().tx.identity.setIdentity({ display: { raw: "Me" } });
  const block = await context.createBlock([
    await context
      .polkadotJs()
      .tx.preimage.notePreimage(call.method.toHex())
      .signAsync(from, { nonce: nonce++ }),
    await context
      .polkadotJs()
      .tx.referenda.submit(
        track === "root" ? { system: "root" } : { Origins: track },
        { Lookup: { Hash: call.hash.toHex(), len: call.method.encodedLength } },
        { After: 1 }
      )
      .signAsync(from, { nonce: nonce++ }),
  ]);
  return expectSubstrateEvent(block, "referenda", "Submitted").data[0].toNumber();
}

export async function cancelProposal(context: DevModeContext, proposal: number) {
  const block = await context.createBlock([
    await context
      .polkadotJs()
      .tx.sudo.sudo(context.polkadotJs().tx.referenda.cancel(proposal))
      .signAsync(alith, { nonce: -1 }),
  ]);
  expectSubstrateEvent(block, "referenda", "Cancelled");
}

export const OPEN_TECHNICAL_COMMITTEE_MEMBERS: KeyringPair[] = [alith, baltathar];
export const OPEN_TECHNICAL_COMMITTEE_THRESHOLD = Math.ceil(
  (OPEN_TECHNICAL_COMMITTEE_MEMBERS.length * 5) / 9
);

export const executeExtViaOpenTechCommittee = async <
  Call extends SubmittableExtrinsic<ApiType>,
  ApiType extends ApiTypes,
>(
  context: DevModeContext,
  extrinsic: Call | string,
  voters: KeyringPair[] = OPEN_TECHNICAL_COMMITTEE_MEMBERS,
  threshold: number = OPEN_TECHNICAL_COMMITTEE_THRESHOLD
) => {
  const openTechCommitteeProposal = context.pjsApi.tx.openTechCommitteeCollective.propose(
    threshold,
    extrinsic,
    100
  );
  const { result: result2 } = await context.createBlock(openTechCommitteeProposal, {
    signer: voters[0],
  });
  if (!result2?.events) {
    throw new Error("No events in block");
  }

  let openTechProposal: `0x${string}` | undefined;
  let openTechProposalIndex: number | undefined;

  filterAndApply(result2.events, "openTechCommitteeCollective", ["Proposed"], (found) => {
    openTechProposalIndex = (found.event as any).data.proposalIndex.toNumber();
    openTechProposal = (found.event as any).data.proposalHash.toHex();
  });

  if (typeof openTechProposal === "undefined" || typeof openTechProposalIndex === "undefined") {
    console.error("Error submitting OpenTechCommittee proposal");
    return result2;
  }

  console.log(
    `🏛️ OpenTechCommittee proposal submitted with proposal id: 
    ${openTechProposalIndex} and hash: ${openTechProposal.slice(0, 6)}...${openTechProposal.slice(
      -4
    )}`
  );

  // Vote on it
  for (const voter of voters) {
    const nonce = (await context.pjsApi.query.system.account(voter.address)).nonce.toNumber();
    const vote = context.pjsApi.tx.openTechCommitteeCollective
      .vote(openTechProposal, openTechProposalIndex, true)
      .signAsync(voter, { nonce });

    await context.createBlock(vote);
  }

  // Close proposal
  const { result } = await context.createBlock(
    [
      context.pjsApi.tx.openTechCommitteeCollective.close(
        openTechProposal,
        openTechProposalIndex,
        {
          refTime: 2_000_000_000,
          proofSize: 100_000,
        },
        100
      ),
    ],
    { signer: voters[0] }
  );

  if (!result) {
    throw new Error("No result in block");
  }

  return result[0];
};

import { ApiPromise, Keyring } from "@polkadot/api";
import { ApiTypes, SubmittableExtrinsic } from "@polkadot/api/types";
import { PalletDemocracyReferendumInfo } from "@polkadot/types/lookup";
import { KeyringPair } from "@polkadot/keyring/types";
import { blake2AsHex } from "@polkadot/util-crypto";
import {
  ALITH_PRIV_KEY,
  BALTATHAR_PRIV_KEY,
  CHARLETH_PRIV_KEY,
  DOROTHY_PRIV_KEY,
} from "./constants";
import { DevTestContext } from "./setup-dev-tests";
import { createBlockWithExtrinsic } from "./substrate-rpc";

const keyring = new Keyring({ type: "ethereum" });

export const notePreimage = async <
  Call extends SubmittableExtrinsic<ApiType>,
  ApiType extends ApiTypes
>(
  context: DevTestContext,
  proposal: Call,
  account: KeyringPair
): Promise<string> => {
  const encodedProposal = proposal.method.toHex() || "";
  await context.polkadotApi.tx.democracy.notePreimage(encodedProposal).signAndSend(account);
  await context.createBlock();

  return blake2AsHex(encodedProposal);
};

export const execFromTwoThirdsOfCouncil = async <
  Call extends SubmittableExtrinsic<ApiType>,
  ApiType extends ApiTypes
>(
  context: DevTestContext,
  polkadotCall: Call
) => {
  // Council members
  const charleth = await keyring.addFromUri(CHARLETH_PRIV_KEY, null, "ethereum");
  const dorothy = await keyring.addFromUri(DOROTHY_PRIV_KEY, null, "ethereum");

  // Charleth submit the proposal to the council (and therefore implicitly votes for)
  let lengthBound = polkadotCall.encodedLength;
  const { events: proposalEvents } = await createBlockWithExtrinsic(
    context,
    charleth,
    context.polkadotApi.tx.councilCollective.propose(2, polkadotCall, lengthBound)
  );
  const proposalHash = proposalEvents
    .find((e) => e.method.toString() == "Proposed")
    .data[2].toHex() as string;

  // Dorothy vote for this proposal and close it
  await Promise.all([
    context.polkadotApi.tx.councilCollective.vote(proposalHash, 0, true).signAndSend(charleth),
    context.polkadotApi.tx.councilCollective.vote(proposalHash, 0, true).signAndSend(dorothy),
  ]);
  await context.createBlock();

  return await createBlockWithExtrinsic(
    context,
    dorothy,
    context.polkadotApi.tx.councilCollective.close(proposalHash, 0, 1_000_000_000, lengthBound)
  );
};

export const execFromAllMembersOfTechCommittee = async <
  Call extends SubmittableExtrinsic<ApiType>,
  ApiType extends ApiTypes
>(
  context: DevTestContext,
  polkadotCall: Call
) => {
  // Tech committee members
  const alith = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
  const baltathar = await keyring.addFromUri(BALTATHAR_PRIV_KEY, null, "ethereum");

  // Alith submit the proposal to the council (and therefore implicitly votes for)
  let lengthBound = polkadotCall.encodedLength;
  const { events: proposalEvents } = await createBlockWithExtrinsic(
    context,
    alith,
    context.polkadotApi.tx.techCommitteeCollective.propose(2, polkadotCall, lengthBound)
  );
  const proposalHash = proposalEvents
    .find((e) => e.method.toString() == "Proposed")
    .data[2].toHex() as string;

  // Get proposal count
  const proposalCount = await context.polkadotApi.query.techCommitteeCollective.proposalCount();

  // Alith, Baltathar vote for this proposal and close it
  await Promise.all([
    context.polkadotApi.tx.techCommitteeCollective
      .vote(proposalHash, Number(proposalCount) - 1, true)
      .signAndSend(alith),
    context.polkadotApi.tx.techCommitteeCollective
      .vote(proposalHash, Number(proposalCount) - 1, true)
      .signAndSend(baltathar),
  ]);

  await context.createBlock();
  await context.createBlock();
  return await createBlockWithExtrinsic(
    context,
    baltathar,
    context.polkadotApi.tx.techCommitteeCollective.close(
      proposalHash,
      Number(proposalCount) - 1,
      1_000_000_000,
      lengthBound
    )
  );
};

export const executeProposalWithCouncil = async (api: ApiPromise, encodedHash: string) => {
  const alith = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
  let nonce = (await api.rpc.system.accountNextIndex(alith.address)).toNumber();
  let referendumNextIndex = (await api.query.democracy.referendumCount()).toNumber();

  // process.stdout.write(
  //   `Sending council motion (${encodedHash} ` +
  //     `[threashold: 1, expected referendum: ${referendumNextIndex}])...`
  // );
  let external = api.tx.democracy.externalProposeMajority(encodedHash);
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

  const alith = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
  let nonce = (await api.rpc.system.accountNextIndex(alith.address)).toNumber();
  await api.tx.democracy.notePreimage(encodedProposal).signAndSend(alith, { nonce: nonce++ });

  await executeProposalWithCouncil(api, encodedHash);
};

import { Keyring } from "@polkadot/api";
import { ApiTypes, SubmittableExtrinsic } from "@polkadot/api/types";
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
  const proposalHash = proposalEvents[0].data[2].toHuman() as string;

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
  const proposalHash = proposalEvents[0].data[2].toHuman() as string;

  // Alith, Baltathar vote for this proposal and close it
  await Promise.all([
    context.polkadotApi.tx.techCommitteeCollective.vote(proposalHash, 0, true).signAndSend(alith),
    context.polkadotApi.tx.techCommitteeCollective
      .vote(proposalHash, 0, true)
      .signAndSend(baltathar),
  ]);

  await context.createBlock();
  await context.createBlock();
  return await createBlockWithExtrinsic(
    context,
    baltathar,
    context.polkadotApi.tx.techCommitteeCollective.close(
      proposalHash,
      0,
      1_000_000_000,
      lengthBound
    )
  );
};

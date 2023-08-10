import { DevModeContext } from "@moonwall/cli";
import { ALITH_ADDRESS, alith } from "@moonwall/util";
import { expectSubstrateEvent } from "./expect.js";
import { ContractCall } from "./contract-call.js";

export class ConvictionVoting extends ContractCall {
  constructor(context: DevModeContext) {
    super("ConvictionVoting", context);
  }

  async voteYes(proposalIndex: number, amount: bigint, conviction: bigint) {
    return await this.callExtrinsic("voteYes", [proposalIndex, amount, conviction]);
  }

  async voteNo(proposalIndex: number, amount: bigint, conviction: bigint) {
    return await this.callExtrinsic("voteNo", [proposalIndex, amount, conviction]);
  }

  async removeVote(proposalIndex: number) {
    return await this.callExtrinsic("removeVote", [proposalIndex]);
  }

  async removeVoteForTrack(proposalIndex: number, trackId: number) {
    return await this.callExtrinsic("removeVoteForTrack", [proposalIndex, trackId]);
  }

  async removeOtherVote(target: `0x${string}`, trackId: number, proposalIndex: number) {
    return await this.callExtrinsic("removeOtherVote", [target, trackId, proposalIndex]);
  }

  async voteSplit(proposalIndex: number, ayes: bigint, nays: bigint) {
    return await this.callExtrinsic("voteSplit", [proposalIndex, ayes, nays]);
  }

  async voteSplitAbstain(proposalIndex: number, ayes: bigint, nays: bigint, abstain: bigint) {
    return await this.callExtrinsic("voteSplitAbstain", [proposalIndex, ayes, nays, abstain]);
  }

  async votingFor(who: `0x${string}`, proposalIndex: number) {
    return await this.callQuery("votingFor", [who, proposalIndex]);
  }

  async classLocksFor(who: `0x${string}`) {
    return await this.callQuery("classLocksFor", [who]);
  }

  async delegate(trackId: number, target: `0x${string}`, conviction: bigint, amount: bigint) {
    return await this.callExtrinsic("delegate", [trackId, target, conviction, amount]);
  }

  async undelegate(trackId: number) {
    return await this.callExtrinsic("undelegate", [trackId]);
  }
}

export async function createProposal(context: DevModeContext, track = "root") {
  let nonce = (await context.polkadotJs().rpc.system.accountNextIndex(ALITH_ADDRESS)).toNumber();
  const call = context.polkadotJs().tx.identity.setIdentity({ display: { raw: "Me" } });
  const block = await context.createBlock([
    await context
      .polkadotJs()
      .tx.preimage.notePreimage(call.toHex())
      .signAsync(alith, { nonce: nonce++ }),
    await context
      .polkadotJs()
      .tx.referenda.submit(
        track == "root" ? { system: "root" } : { Origins: track },
        { Lookup: { Hash: call.hash.toHex(), len: call.length } },
        { After: 1 }
      )
      .signAsync(alith, { nonce: nonce++ }),
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

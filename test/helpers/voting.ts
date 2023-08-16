import { DevModeContext } from "@moonwall/cli";
import { ALITH_ADDRESS, alith } from "@moonwall/util";
import { expectSubstrateEvent } from "./expect.js";

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

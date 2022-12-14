import "@moonbeam-network/api-augment";

import { blake2AsHex } from "@polkadot/util-crypto";
import { expect } from "chai";

import { alith } from "../../util/accounts";
import { MICROGLMR } from "../../util/constants";
import { notePreimage } from "../../util/governance";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { u8aToHex } from "@polkadot/util";

describeDevMoonbeam("Democracy - Preimage", (context) => {
  it("should be notable", async function () {
    const encodedProposal =
      context.polkadotApi.tx.parachainStaking
        .setParachainBondAccount(alith.address)
        .method.toHex() || "";
    const encodedHash = blake2AsHex(encodedProposal);
    await context.createBlock(context.polkadotApi.tx.preimage.notePreimage(encodedProposal));

    const preimageStatus = (await context.polkadotApi.query.preimage.statusFor(encodedHash)) as any;
    expect(preimageStatus.isSome).to.be.true;
    expect(preimageStatus.unwrap().isUnrequested).to.be.true;

    const [proposer, balance] = preimageStatus.unwrap().asUnrequested.deposit;
    expect(u8aToHex(proposer)).to.eq(alith.address.toLowerCase());
    expect(balance.toBigInt()).to.eq(5002200n * MICROGLMR);
  });
});

describeDevMoonbeam("Democracy - Preimage", (context) => {
  it("should be forgettable immediatly", async function () {
    const encodedHash = await notePreimage(
      context,
      context.polkadotApi.tx.parachainStaking.setParachainBondAccount(alith.address),
      alith
    );

    await context.createBlock(context.polkadotApi.tx.preimage.unnotePreimage(encodedHash));

    const preimageStatus = (await context.polkadotApi.query.preimage.statusFor(encodedHash)) as any;
    expect(preimageStatus.isSome).to.be.false;
  });
});

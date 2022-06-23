import "@moonbeam-network/api-augment";

import { blake2AsHex } from "@polkadot/util-crypto";
import { expect } from "chai";

import { alith } from "../../util/accounts";
import { MICROGLMR } from "../../util/constants";
import { notePreimage } from "../../util/governance";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";

describeDevMoonbeam("Democracy - Preimage", (context) => {
  it("should be notable", async function () {
    const encodedProposal =
      context.polkadotApi.tx.parachainStaking
        .setParachainBondAccount(alith.address)
        .method.toHex() || "";
    const encodedHash = blake2AsHex(encodedProposal);
    await context.createBlock(context.polkadotApi.tx.democracy.notePreimage(encodedProposal));

    const preimageStatus = await context.polkadotApi.query.democracy.preimages(encodedHash);
    expect(preimageStatus.isSome).to.be.true;
    expect(preimageStatus.unwrap().isAvailable).to.eq(true, "Preimage should be available");
    expect(preimageStatus.unwrap().asAvailable.provider.toString()).to.equal(alith.address);
    expect(preimageStatus.unwrap().asAvailable.deposit.toBigInt()).to.equal(2200n * MICROGLMR);
  });
});

describeDevMoonbeam("Democracy - Preimage", (context) => {
  it("should not be forgettable immediately", async function () {
    const encodedHash = await notePreimage(
      context,
      context.polkadotApi.tx.parachainStaking.setParachainBondAccount(alith.address),
      alith
    );

    const {
      result: { error },
    } = await context.createBlock(
      context.polkadotApi.tx.democracy.reapPreimage(encodedHash, 10000)
    );

    expect(error.name).to.equal("TooEarly");

    const preimageStatus = await context.polkadotApi.query.democracy.preimages(encodedHash);
    expect(preimageStatus.isSome).to.be.true;
  });
});

describeDevMoonbeam("Democracy - Preimage", (context) => {
  it("should be forgettable after voting period", async function () {
    const encodedHash = await notePreimage(
      context,
      context.polkadotApi.tx.parachainStaking.setParachainBondAccount(alith.address),
      alith
    );

    await context.createBlock(context.polkadotApi.tx.democracy.reapPreimage(encodedHash, 10000));

    const preimageStatus = await context.polkadotApi.query.democracy.preimages(encodedHash);
    expect(preimageStatus.isSome).to.be.true;
    expect(preimageStatus.unwrap().isAvailable).to.eq(true, "Preimage should be available");
    expect(preimageStatus.unwrap().asAvailable.provider.toString()).to.equal(alith.address);
    expect(preimageStatus.unwrap().asAvailable.deposit.toBigInt()).to.equal(2200n * MICROGLMR);
  });
});

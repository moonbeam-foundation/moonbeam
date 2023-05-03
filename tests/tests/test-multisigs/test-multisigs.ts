import "@polkadot/api-augment";
import "@moonbeam-network/api-augment";
import { blake2AsHex, createKeyMulti } from "@polkadot/util-crypto";
import { u8aToHex } from "@polkadot/util";
import { expect } from "chai";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import {
  alith,
  ALITH_ADDRESS,
  DOROTHY_ADDRESS,
  CHARLETH_ADDRESS,
  BALTATHAR_ADDRESS,
  baltathar,
} from "../../util/accounts";

describeDevMoonbeam("Multisigs - perform multisigs operations", (context) => {
  let threshold: number;
  let call: any;
  let encodedCall: string;
  let encodedCallHash: string;

  // multisig accountId
  let encodedMultisigId: Uint8Array;
  let multisigId: string;

  before("Should create a multisigId and set call values", async function () {
    // set threshold and create multisig accountId
    threshold = 2;
    encodedMultisigId = createKeyMulti([ALITH_ADDRESS, BALTATHAR_ADDRESS, CHARLETH_ADDRESS], 2);
    multisigId = u8aToHex(encodedMultisigId.slice(0, 20));

    // encode and hash the call we want to dispatch as a multisig operation
    call = context.polkadotApi.tx.balances.transferKeepAlive(DOROTHY_ADDRESS, 20);
    encodedCall = call.method.toHex();
    encodedCallHash = blake2AsHex(encodedCall);
  });

  it("Should create a multisig operation with asMulti", async () => {
    // set signatories
    const otherSignatories = [BALTATHAR_ADDRESS, CHARLETH_ADDRESS];
    const block = await context.createBlock(
      context.polkadotApi.tx.multisig
        .asMulti(threshold, otherSignatories, null, encodedCall, {})
        .signAsync(alith)
    );

    // check the event 'NewMultisig' was emitted
    const records = (await context.polkadotApi.query.system.events()) as any;
    const events = records.filter(
      ({ event }) => event.section == "multisig" && event.method == "NewMultisig"
    );
    expect(events).to.have.lengthOf(1);
    expect(block.result.successful).to.be.true;
  });

  it("Should be able to approve a multisig operation with approveAsMulti", async function () {
    // signatories (sorted)
    const otherSignatories = [CHARLETH_ADDRESS, ALITH_ADDRESS];
    // create a new multisig operation
    await context.createBlock(
      context.polkadotApi.tx.multisig
        .asMulti(threshold, otherSignatories, null, encodedCall, {})
        .signAsync(alith)
    );

    // take the info of the new multisig operation saved in storage
    const multisigInfo = await context.polkadotApi.query.multisig.multisigs(
      multisigId,
      encodedCallHash
    );
    const block = await context.createBlock(
      context.polkadotApi.tx.multisig
        .approveAsMulti(
          threshold,
          otherSignatories,
          multisigInfo.toHuman()["when"],
          encodedCallHash,
          {}
        )
        .signAsync(baltathar)
    );

    // check the event 'MultisigApproval' was emitted
    const records = (await context.polkadotApi.query.system.events()) as any;
    const events = records.filter(
      ({ event }) => event.section == "multisig" && event.method == "MultisigApproval"
    );
    expect(events).to.have.lengthOf(1);
    expect(block.result.successful).to.be.true;
  });

  it("Should be able to cancel a multisig operation", async () => {
    // signatories (sorted)
    const otherSignatories = [BALTATHAR_ADDRESS, CHARLETH_ADDRESS];
    // create a new multisig operation
    await context.createBlock(
      context.polkadotApi.tx.multisig
        .asMulti(threshold, otherSignatories, null, encodedCall, {})
        .signAsync(alith)
    );

    // take the info of the new multisig operation saved in storage
    const multisigInfo = await context.polkadotApi.query.multisig.multisigs(
      multisigId,
      encodedCallHash
    );
    const block = await context.createBlock(
      context.polkadotApi.tx.multisig
        .cancelAsMulti(threshold, otherSignatories, multisigInfo.toHuman()["when"], encodedCallHash)
        .signAsync(alith)
    );

    // check the event 'MultisigCancelled' was emitted
    const records = (await context.polkadotApi.query.system.events()) as any;
    const events = records.filter(
      ({ event }) => event.section == "multisig" && event.method == "MultisigCancelled"
    );
    expect(events).to.have.lengthOf(1);
    expect(block.result.successful).to.be.true;
  });

  it("Should fail if signatories are out of order", async () => {
    // signatories (they are not sorted)
    const otherSignatories = [CHARLETH_ADDRESS, BALTATHAR_ADDRESS];
    const block = await context.createBlock(
      context.polkadotApi.tx.multisig
        .asMulti(threshold, otherSignatories, null, encodedCall, {})
        .signAsync(alith)
    );
    expect(block.result.error.name).to.equal("SignatoriesOutOfOrder");
    expect(block.result.successful).to.be.false;
  });

  it("Should fail if sender is present in signatories", async () => {
    // signatories (with sender in signatories)
    const otherSignatories = [ALITH_ADDRESS, BALTATHAR_ADDRESS];
    const block = await context.createBlock(
      context.polkadotApi.tx.multisig
        .asMulti(threshold, otherSignatories, null, encodedCall, {})
        .signAsync(alith)
    );
    expect(block.result.error.name).to.equal("SenderInSignatories");
    expect(block.result.successful).to.be.false;
  });
});

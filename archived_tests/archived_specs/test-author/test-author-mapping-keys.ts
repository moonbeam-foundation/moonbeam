import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { charleth, dorothy } from "../../../util/accounts";
import { getBlockExtrinsic } from "../../../util/block";
import { describeDevMoonbeam } from "../../../util/setup-dev-tests";

const debug = require("debug")("test:author-mapping");

// Keys used to set author-mapping in the tests
const originalKeys = [
  "0x0000000000000000000000000000000000000000000000000000000000000001",
  "0x0000000000000000000000000000000000000000000000000000000000000002",
];
// Concatenated keys
const concatOriginalKeys = `0x${originalKeys.map((key) => key.slice(2)).join("")}`;

describeDevMoonbeam("Author Mapping - Set Charlie first time keys", (context) => {
  before("setup account & keys", async function () {
    debug(`Setting account ${charleth.address} keys: ${concatOriginalKeys}`);
    // TODO: fix all setKeys with api 1600.1
    await (context.polkadotApi.tx.authorMapping.setKeys as any as any)(
      concatOriginalKeys
    ).signAndSend(charleth);
    await context.createBlock();
  });

  it("should succeed", async function () {
    const { extrinsic, resultEvent } = await getBlockExtrinsic(
      context.polkadotApi,
      await context.polkadotApi.rpc.chain.getBlockHash(),
      "authorMapping",
      "setKeys"
    );

    expect(extrinsic).to.exist;
    expect(resultEvent.method).to.equal("ExtrinsicSuccess");
  });

  it("should send KeysRegistered event", async function () {
    const { events } = await getBlockExtrinsic(
      context.polkadotApi,
      await context.polkadotApi.rpc.chain.getBlockHash(),
      "authorMapping",
      "setKeys"
    );
    expect(events.find((e) => e.section == "authorMapping" && e.method == "KeysRegistered")).to
      .exist;
  });

  it("should set new keys", async function () {
    const mapping = await context.polkadotApi.query.authorMapping.mappingWithDeposit(
      originalKeys[0]
    );
    expect(mapping.isSome).to.be.true;
    expect(mapping.unwrap().account.toString()).to.equal(charleth.address);
    expect(mapping.unwrap().keys_.toString()).to.equal(originalKeys[1]);
  });

  it("should set correct nimbus lookup", async function () {
    const nimbusLookup = (await context.polkadotApi.query.authorMapping.nimbusLookup(
      charleth.address
    )) as any;
    expect(nimbusLookup.isSome).to.be.true;
    expect(nimbusLookup.unwrap().toString()).to.equal(originalKeys[0]);
  });
});

describeDevMoonbeam("Author Mapping - Update Charlie mapping to the same keys", (context) => {
  before("setup account & keys", async function () {
    await (context.polkadotApi.tx.authorMapping.setKeys as any)(concatOriginalKeys).signAndSend(
      charleth
    );
    await context.createBlock();

    // Updating with the same keys
    await (context.polkadotApi.tx.authorMapping.setKeys as any)(concatOriginalKeys).signAndSend(
      charleth
    );
    await context.createBlock();
  });

  it("should succeed", async function () {
    const { extrinsic, resultEvent } = await getBlockExtrinsic(
      context.polkadotApi,
      await context.polkadotApi.rpc.chain.getBlockHash(),
      "authorMapping",
      "setKeys"
    );

    expect(extrinsic).to.exist;
    expect(resultEvent.method).to.equal("ExtrinsicSuccess");
  });

  it("should send KeysRotated event", async function () {
    const { events } = await getBlockExtrinsic(
      context.polkadotApi,
      await context.polkadotApi.rpc.chain.getBlockHash(),
      "authorMapping",
      "setKeys"
    );
    expect(events.find((e) => e.section == "authorMapping" && e.method == "KeysRotated")).to.exist;
  });

  it("should keep the same keys", async function () {
    const mapping = await context.polkadotApi.query.authorMapping.mappingWithDeposit(
      originalKeys[0]
    );
    expect(mapping.isSome).to.be.true;
    expect(mapping.unwrap().account.toString()).to.equal(charleth.address);
    expect(mapping.unwrap().keys_.toString()).to.equal(originalKeys[1]);
  });

  it("should keep the same nimbus lookup", async function () {
    const nimbusLookup = (await context.polkadotApi.query.authorMapping.nimbusLookup(
      charleth.address
    )) as any;
    expect(nimbusLookup.isSome).to.be.true;
    expect(nimbusLookup.unwrap().toString()).to.equal(originalKeys[0]);
  });
});

describeDevMoonbeam("Author Mapping - Update different keys", (context) => {
  const newKeys = [
    "0x0000000000000000000000000000000000000000000000000000000000000003",
    "0x0000000000000000000000000000000000000000000000000000000000000004",
  ];
  const concatNewKeys = `0x${newKeys.map((key) => key.slice(2)).join("")}`;

  before("setup account & keys", async function () {
    await (context.polkadotApi.tx.authorMapping.setKeys as any)(concatOriginalKeys).signAndSend(
      charleth
    );
    await context.createBlock();

    // Updating with different keys
    await (context.polkadotApi.tx.authorMapping.setKeys as any)(concatNewKeys).signAndSend(
      charleth
    );
    await context.createBlock();
  });

  it("should succeed", async function () {
    const { extrinsic, resultEvent } = await getBlockExtrinsic(
      context.polkadotApi,
      await context.polkadotApi.rpc.chain.getBlockHash(),
      "authorMapping",
      "setKeys"
    );

    expect(extrinsic).to.exist;
    expect(resultEvent.method).to.equal("ExtrinsicSuccess");
  });

  it("should send KeysRotated event", async function () {
    const { events } = await getBlockExtrinsic(
      context.polkadotApi,
      await context.polkadotApi.rpc.chain.getBlockHash(),
      "authorMapping",
      "setKeys"
    );
    expect(events.find((e) => e.section == "authorMapping" && e.method == "KeysRotated")).to.exist;
  });

  it("should remove previous keys", async function () {
    const mapping = await context.polkadotApi.query.authorMapping.mappingWithDeposit(
      originalKeys[0]
    );
    expect(mapping.isNone).to.be.true;
  });

  it("should set new keys", async function () {
    const mapping = await context.polkadotApi.query.authorMapping.mappingWithDeposit(newKeys[0]);
    expect(mapping.isSome).to.be.true;
    expect(mapping.unwrap().account.toString()).to.equal(charleth.address);
    expect(mapping.unwrap().keys_.toString()).to.equal(newKeys[1]);
  });

  it("should set correct nimbus lookup", async function () {
    const nimbusLookup = (await context.polkadotApi.query.authorMapping.nimbusLookup(
      charleth.address
    )) as any;
    expect(nimbusLookup.isSome).to.be.true;
    expect(nimbusLookup.unwrapOr(null)).to.not.equal(null);
    expect(nimbusLookup.unwrap().toString()).to.equal(newKeys[0]);
  });
});

describeDevMoonbeam("Author Mapping - Remove Charlie keys", (context) => {
  before("setup account & keys", async function () {
    await (context.polkadotApi.tx.authorMapping.setKeys as any)(concatOriginalKeys).signAndSend(
      charleth
    );
    await context.createBlock();

    // Remove the keys
    await context.polkadotApi.tx.authorMapping.removeKeys().signAndSend(charleth);
    await context.createBlock();
  });

  it("should succeed", async function () {
    const { extrinsic, resultEvent } = await getBlockExtrinsic(
      context.polkadotApi,
      await context.polkadotApi.rpc.chain.getBlockHash(),
      "authorMapping",
      "removeKeys"
    );

    expect(extrinsic).to.exist;
    expect(resultEvent.method).to.equal("ExtrinsicSuccess");
  });

  it("should send KeysRemoved event", async function () {
    const { events } = await getBlockExtrinsic(
      context.polkadotApi,
      await context.polkadotApi.rpc.chain.getBlockHash(),
      "authorMapping",
      "removeKeys"
    );
    expect(events.find((e) => e.section == "authorMapping" && e.method == "KeysRemoved")).to.exist;
  });

  it("should remove keys", async function () {
    const mapping = await context.polkadotApi.query.authorMapping.mappingWithDeposit(
      originalKeys[0]
    );
    expect(mapping.isNone).to.be.true;
  });

  it("should remove nimbus mapping", async function () {
    const nimbusLookup = (await context.polkadotApi.query.authorMapping.nimbusLookup(
      charleth.address
    )) as any;
    expect(nimbusLookup.isNone).to.be.true;
  });
});

describeDevMoonbeam("Author Mapping - Removing non-existing author", (context) => {
  before("setup account & keys", async function () {
    // Remove the keys
    await context.polkadotApi.tx.authorMapping.removeKeys().signAndSend(dorothy);
    await context.createBlock();
  });

  it("should fail", async function () {
    const { extrinsic, resultEvent } = await getBlockExtrinsic(
      context.polkadotApi,
      await context.polkadotApi.rpc.chain.getBlockHash(),
      "authorMapping",
      "removeKeys"
    );

    expect(extrinsic).to.exist;
    expect(resultEvent.method).to.equal("ExtrinsicFailed");
  });

  it("should not send KeysRemoved event", async function () {
    const { events } = await getBlockExtrinsic(
      context.polkadotApi,
      await context.polkadotApi.rpc.chain.getBlockHash(),
      "authorMapping",
      "removeKeys"
    );
    expect(events.find((e) => e.section == "authorMapping" && e.method == "KeysRemoved")).to.not
      .exist;
  });
});

describeDevMoonbeam("Author Mapping - Update someone else nimbus key", (context) => {
  before("setup account & keys", async function () {
    await (context.polkadotApi.tx.authorMapping.setKeys as any)(concatOriginalKeys).signAndSend(
      charleth
    );
    await context.createBlock();

    // Setting same key but with ethan
    await (context.polkadotApi.tx.authorMapping.setKeys as any)(concatOriginalKeys).signAndSend(
      dorothy
    );
    await context.createBlock();
  });

  it("should fail", async function () {
    const { extrinsic, resultEvent } = await getBlockExtrinsic(
      context.polkadotApi,
      await context.polkadotApi.rpc.chain.getBlockHash(),
      "authorMapping",
      "setKeys"
    );

    expect(extrinsic).to.exist;
    expect(resultEvent.method).to.equal("ExtrinsicFailed");
  });

  it("should not send any authorMapping event", async function () {
    const { events } = await getBlockExtrinsic(
      context.polkadotApi,
      await context.polkadotApi.rpc.chain.getBlockHash(),
      "authorMapping",
      "removeKeys"
    );
    expect(events.find((e) => e.section == "authorMapping")).to.not.exist;
  });

  it("should keep the same keys to Faith", async function () {
    const mapping = await context.polkadotApi.query.authorMapping.mappingWithDeposit(
      originalKeys[0]
    );
    expect(mapping.isSome).to.be.true;
    expect(mapping.unwrap().account.toString()).to.equal(charleth.address);
    expect(mapping.unwrap().keys_.toString()).to.equal(originalKeys[1]);
  });

  it("should not set nimbus lookup to Ethan", async function () {
    const nimbusLookup = (await context.polkadotApi.query.authorMapping.nimbusLookup(
      dorothy.address
    )) as any;
    expect(nimbusLookup.isNone).to.be.true;
  });

  it("should keep the same nimbus lookup to Faith", async function () {
    const nimbusLookup = (await context.polkadotApi.query.authorMapping.nimbusLookup(
      charleth.address
    )) as any;
    expect(nimbusLookup.isSome).to.be.true;
    expect(nimbusLookup.unwrap().toString()).to.equal(originalKeys[0]);
  });
});

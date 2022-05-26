import { expect } from "chai";
import Keyring from "@polkadot/keyring";

import { ETHAN_PRIVATE_KEY, FAITH_PRIVATE_KEY } from "../../util/constants";
import { describeDevMoonbeam, DevTestContext } from "../../util/setup-dev-tests";
import { getBlockExtrinsic } from "../../util/block";
import { sendPrecompileTx } from "../../util/transactions";
const debug = require("debug")("test-precompile:author-mapping");
const keyring = new Keyring({ type: "ethereum" });

// Keys used to set author-mapping in the tests
const originalKeys = [
  "0x0000000000000000000000000000000000000000000000000000000000000001",
  "0x0000000000000000000000000000000000000000000000000000000000000002",
];
// Concatenated keys
const concatOriginalKeys = `0x${originalKeys.map((key) => key.slice(2)).join("")}`;

// We are using Faith account because she doesn't have authorMapping setup at genesis
const faith = keyring.addFromUri(FAITH_PRIVATE_KEY, null, "ethereum");

const AUTHOR_MAPPING_PRECOMPILE_ADDRESS = "0x0000000000000000000000000000000000000807";
const SELECTORS = {
  set_keys: "47f92fc4",
  remove_keys: "3b6c4284",
};

const setKeysThroughPrecompile = async (
  context: DevTestContext,
  account: string,
  private_key: string,
  keys: string
) => {
  await sendPrecompileTx(
    context,
    AUTHOR_MAPPING_PRECOMPILE_ADDRESS,
    SELECTORS,
    account,
    private_key,
    "set_keys",
    [keys]
  );
};

describeDevMoonbeam("Precompile Author Mapping - Set Faith first time keys", (context) => {
  before("setup account & keys", async function () {
    debug(`Setting account ${faith.address} keys: ${concatOriginalKeys}`);
    await setKeysThroughPrecompile(context, faith.address, FAITH_PRIVATE_KEY, concatOriginalKeys);
  });

  it("should succeed", async function () {
    const { extrinsic, events, resultEvent } = await getBlockExtrinsic(
      context.polkadotApi,
      await context.polkadotApi.rpc.chain.getBlockHash(),
      "ethereum",
      "transact"
    );

    expect(extrinsic).to.exist;
    expect(resultEvent.method).to.equal("ExtrinsicSuccess");
    expect(
      (events.find((e) => e.section == "ethereum" && e.method == "Executed").data[3] as any)
        .isSucceed
    ).to.be.true;
  });

  it("should send KeysRegistered event", async function () {
    const { events } = await getBlockExtrinsic(
      context.polkadotApi,
      await context.polkadotApi.rpc.chain.getBlockHash(),
      "ethereum",
      "transact"
    );
    expect(events.find((e) => e.section == "authorMapping" && e.method == "KeysRegistered")).to
      .exist;
  });

  it("should set new keys", async function () {
    const mapping = (await context.polkadotApi.query.authorMapping.mappingWithDeposit(
      originalKeys[0]
    )) as any;
    expect(mapping.isSome).to.be.true;
    expect(mapping.unwrap().account.toString()).to.equal(faith.address);
    expect(mapping.unwrap().keys_.toString()).to.equal(originalKeys[1]);
  });

  it("should set correct nimbus lookup", async function () {
    const nimbusLookup = (await context.polkadotApi.query.authorMapping.nimbusLookup(
      faith.address
    )) as any;
    expect(nimbusLookup.isSome).to.be.true;
    expect(nimbusLookup.unwrap().toString()).to.equal(originalKeys[0]);
  });
});

describeDevMoonbeam(
  "Precompile Author Mapping - Update Faith mapping to the same keys",
  (context) => {
    before("setup account & keys", async function () {
      await setKeysThroughPrecompile(context, faith.address, FAITH_PRIVATE_KEY, concatOriginalKeys);

      // Updating with the same keys
      await setKeysThroughPrecompile(context, faith.address, FAITH_PRIVATE_KEY, concatOriginalKeys);
    });

    it("should succeed", async function () {
      const { extrinsic, events, resultEvent } = await getBlockExtrinsic(
        context.polkadotApi,
        await context.polkadotApi.rpc.chain.getBlockHash(),
        "ethereum",
        "transact"
      );

      expect(extrinsic).to.exist;
      expect(resultEvent.method).to.equal("ExtrinsicSuccess");
      expect(
        (events.find((e) => e.section == "ethereum" && e.method == "Executed").data[3] as any)
          .isSucceed
      ).to.be.true;
    });

    it("should send KeysRotated event", async function () {
      const { events } = await getBlockExtrinsic(
        context.polkadotApi,
        await context.polkadotApi.rpc.chain.getBlockHash(),
        "ethereum",
        "transact"
      );
      expect(events.find((e) => e.section == "authorMapping" && e.method == "KeysRotated")).to
        .exist;
    });

    it("should keep the same keys", async function () {
      const mapping = (await context.polkadotApi.query.authorMapping.mappingWithDeposit(
        originalKeys[0]
      )) as any;
      expect(mapping.isSome).to.be.true;
      expect(mapping.unwrap().account.toString()).to.equal(faith.address);
      expect(mapping.unwrap().keys_.toString()).to.equal(originalKeys[1]);
    });

    it("should keep the same nimbus lookup", async function () {
      const nimbusLookup = (await context.polkadotApi.query.authorMapping.nimbusLookup(
        faith.address
      )) as any;
      expect(nimbusLookup.isSome).to.be.true;
      expect(nimbusLookup.unwrap().toString()).to.equal(originalKeys[0]);
    });
  }
);

describeDevMoonbeam("Precompile Author Mapping - Update different keys", (context) => {
  const newKeys = [
    "0x0000000000000000000000000000000000000000000000000000000000000003",
    "0x0000000000000000000000000000000000000000000000000000000000000004",
  ];
  const concatNewKeys = `0x${newKeys.map((key) => key.slice(2)).join("")}`;

  before("setup account & keys", async function () {
    await setKeysThroughPrecompile(context, faith.address, FAITH_PRIVATE_KEY, concatOriginalKeys);

    // Updating with different keys
    await setKeysThroughPrecompile(context, faith.address, FAITH_PRIVATE_KEY, concatNewKeys);
  });

  it("should succeed", async function () {
    const { extrinsic, events, resultEvent } = await getBlockExtrinsic(
      context.polkadotApi,
      await context.polkadotApi.rpc.chain.getBlockHash(),
      "ethereum",
      "transact"
    );

    expect(extrinsic).to.exist;
    expect(resultEvent.method).to.equal("ExtrinsicSuccess");
    expect(
      (events.find((e) => e.section == "ethereum" && e.method == "Executed").data[3] as any)
        .isSucceed
    ).to.be.true;
  });

  it("should send KeysRotated event", async function () {
    const { events } = await getBlockExtrinsic(
      context.polkadotApi,
      await context.polkadotApi.rpc.chain.getBlockHash(),
      "ethereum",
      "transact"
    );
    expect(events.find((e) => e.section == "authorMapping" && e.method == "KeysRotated")).to.exist;
  });

  it("should remove previous keys", async function () {
    const mapping = (await context.polkadotApi.query.authorMapping.mappingWithDeposit(
      originalKeys[0]
    )) as any;
    expect(mapping.isNone).to.be.true;
  });

  it("should set new keys", async function () {
    const mapping = (await context.polkadotApi.query.authorMapping.mappingWithDeposit(
      newKeys[0]
    )) as any;
    expect(mapping.isSome).to.be.true;
    expect(mapping.unwrap().account.toString()).to.equal(faith.address);
    expect(mapping.unwrap().keys_.toString()).to.equal(newKeys[1]);
  });

  it("should set correct nimbus lookup", async function () {
    const nimbusLookup = (await context.polkadotApi.query.authorMapping.nimbusLookup(
      faith.address
    )) as any;
    expect(nimbusLookup.isSome).to.be.true;
    expect(nimbusLookup.unwrapOr(null)).to.not.equal(null);
    expect(nimbusLookup.unwrap().toString()).to.equal(newKeys[0]);
  });
});

describeDevMoonbeam("Precompile Author Mapping - Remove Faith keys", (context) => {
  before("setup account & keys", async function () {
    await setKeysThroughPrecompile(context, faith.address, FAITH_PRIVATE_KEY, concatOriginalKeys);

    // Remove the keys
    await sendPrecompileTx(
      context,
      AUTHOR_MAPPING_PRECOMPILE_ADDRESS,
      SELECTORS,
      faith.address,
      FAITH_PRIVATE_KEY,
      "remove_keys",
      []
    );
  });

  it("should succeed", async function () {
    const { extrinsic, events, resultEvent } = await getBlockExtrinsic(
      context.polkadotApi,
      await context.polkadotApi.rpc.chain.getBlockHash(),
      "ethereum",
      "transact"
    );

    expect(extrinsic).to.exist;
    expect(resultEvent.method).to.equal("ExtrinsicSuccess");
    expect(
      (events.find((e) => e.section == "ethereum" && e.method == "Executed").data[3] as any)
        .isSucceed
    ).to.be.true;
  });

  it("should send KeysRemoved event", async function () {
    const { events } = await getBlockExtrinsic(
      context.polkadotApi,
      await context.polkadotApi.rpc.chain.getBlockHash(),
      "ethereum",
      "transact"
    );
    expect(events.find((e) => e.section == "authorMapping" && e.method == "KeysRemoved")).to.exist;
  });

  it("should remove keys", async function () {
    const mapping = (await context.polkadotApi.query.authorMapping.mappingWithDeposit(
      originalKeys[0]
    )) as any;
    expect(mapping.isNone).to.be.true;
  });

  it("should remove nimbus mapping", async function () {
    const nimbusLookup = (await context.polkadotApi.query.authorMapping.nimbusLookup(
      faith.address
    )) as any;
    expect(nimbusLookup.isNone).to.be.true;
  });
});

describeDevMoonbeam("Precompile Author Mapping - Removing non-existing author", (context) => {
  // Using ethan who doesn't have author-mapping

  before("setup account & keys", async function () {
    // Remove the keys
    await sendPrecompileTx(
      context,
      AUTHOR_MAPPING_PRECOMPILE_ADDRESS,
      SELECTORS,
      faith.address,
      FAITH_PRIVATE_KEY,
      "remove_keys",
      []
    );
  });

  it("should revert", async function () {
    const { extrinsic, events, resultEvent } = await getBlockExtrinsic(
      context.polkadotApi,
      await context.polkadotApi.rpc.chain.getBlockHash(),
      "ethereum",
      "transact"
    );

    expect(extrinsic).to.exist;
    // ethereum revert is still a successful substrate extrinsic
    expect(resultEvent.method).to.equal("ExtrinsicSuccess");
    expect(
      (events.find((e) => e.section == "ethereum" && e.method == "Executed").data[3] as any)
        .isRevert
    ).to.be.true;
  });

  it("should not send KeysRemoved event", async function () {
    const { events } = await getBlockExtrinsic(
      context.polkadotApi,
      await context.polkadotApi.rpc.chain.getBlockHash(),
      "ethereum",
      "transact"
    );
    expect(events.find((e) => e.section == "authorMapping" && e.method == "KeysRemoved")).to.not
      .exist;
  });
});

describeDevMoonbeam("Precompile Author Mapping - Update someone else nimbus key", (context) => {
  // Using ethan who doesn't have author-mapping
  const ethan = keyring.addFromUri(ETHAN_PRIVATE_KEY, null, "ethereum");

  before("setup account & keys", async function () {
    await setKeysThroughPrecompile(context, faith.address, FAITH_PRIVATE_KEY, concatOriginalKeys);

    // Setting same key but with ethan
    await setKeysThroughPrecompile(context, ethan.address, ETHAN_PRIVATE_KEY, concatOriginalKeys);
  });

  it("should revert", async function () {
    const { extrinsic, events, resultEvent } = await getBlockExtrinsic(
      context.polkadotApi,
      await context.polkadotApi.rpc.chain.getBlockHash(),
      "ethereum",
      "transact"
    );

    expect(extrinsic).to.exist;
    // ethereum revert is still a successful substrate extrinsic
    expect(resultEvent.method).to.equal("ExtrinsicSuccess");
    expect(
      (events.find((e) => e.section == "ethereum" && e.method == "Executed").data[3] as any)
        .isRevert
    ).to.be.true;
  });

  it("should not send any authorMapping event", async function () {
    const { events } = await getBlockExtrinsic(
      context.polkadotApi,
      await context.polkadotApi.rpc.chain.getBlockHash(),
      "ethereum",
      "transact"
    );
    expect(events.find((e) => e.section == "authorMapping")).to.not.exist;
  });

  it("should keep the same keys to Faith", async function () {
    const mapping = (await context.polkadotApi.query.authorMapping.mappingWithDeposit(
      originalKeys[0]
    )) as any;
    expect(mapping.isSome).to.be.true;
    expect(mapping.unwrap().account.toString()).to.equal(faith.address);
    expect(mapping.unwrap().keys_.toString()).to.equal(originalKeys[1]);
  });

  it("should not set nimbus lookup to Ethan", async function () {
    const nimbusLookup = (await context.polkadotApi.query.authorMapping.nimbusLookup(
      ethan.address
    )) as any;
    expect(nimbusLookup.isNone).to.be.true;
  });

  it("should keep the same nimbus lookup to Faith", async function () {
    const nimbusLookup = (await context.polkadotApi.query.authorMapping.nimbusLookup(
      faith.address
    )) as any;
    expect(nimbusLookup.isSome).to.be.true;
    expect(nimbusLookup.unwrap().toString()).to.equal(originalKeys[0]);
  });
});

// Testing invalid inputs

describeDevMoonbeam("Precompile Author Mapping - Set Faith only 1 key", (context) => {
  it("should fail", async function () {
    await setKeysThroughPrecompile(context, faith.address, FAITH_PRIVATE_KEY, originalKeys[0]);
    const { extrinsic, events, resultEvent } = await getBlockExtrinsic(
      context.polkadotApi,
      await context.polkadotApi.rpc.chain.getBlockHash(),
      "ethereum",
      "transact"
    );

    expect(extrinsic).to.exist;
    expect(resultEvent.method).to.equal("ExtrinsicFailed");
    expect(
      (events.find((e) => e.section == "ethereum" && e.method == "Executed").data[3] as any)
        .isRevert
    ).to.be.true;
  });
});

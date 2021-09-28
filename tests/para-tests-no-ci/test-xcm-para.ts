import Keyring from "@polkadot/keyring";
import { expect } from "chai";
import { BN, hexToU8a } from "@polkadot/util";

import { ALITH, ALITH_PRIV_KEY, BALTATHAR, BALTATHAR_PRIV_KEY } from "../util/constants";
import { describeParachain } from "../util/setup-para-tests";
import { createBlockWithExtrinsicParachain, logEvents, waitOneBlock } from "../util/substrate-rpc";
import { KeyringPair } from "@polkadot/keyring/types";
import { ApiPromise } from "@polkadot/api";

const palletId = "0x6D6f646c617373746d6E67720000000000000000";
const HUNDRED_UNITS = 100000000000000;
const THOUSAND_UNITS = 1000000000000000;

const relayAssetMetadata = {
  name: "DOT",
  symbol: "DOT",
  decimals: new BN(12),
  isFrozen: false,
};
const paraAssetMetadata = {
  name: "GLMR",
  symbol: "GLMR",
  decimals: new BN(18),
  isFrozen: false,
};
const sourceLocation = { XCM: { X1: "Parent" } };

async function registerAssetToParachain(parachainApi: ApiPromise, sudoKeyring: KeyringPair) {
  const { events: eventsRegister } = await createBlockWithExtrinsicParachain(
    parachainApi,
    sudoKeyring,
    parachainApi.tx.sudo.sudo(
      parachainApi.tx.assetManager.registerAsset(sourceLocation, relayAssetMetadata, new BN(1))
    )
  );
  let assetId: string;
  // Look for assetId in events
  eventsRegister.forEach((e) => {
    let ev = e.toHuman();
    if (ev.section === "assetManager") {
      assetId = ev.data[0];
    }
  });
  if (!assetId) {
    await new Promise((res) => setTimeout(res, 20000));
  }
  assetId = assetId.replace(/,/g, "");

  // setAssetUnitsPerSecond
  const { events } = await createBlockWithExtrinsicParachain(
    parachainApi,
    sudoKeyring,
    parachainApi.tx.sudo.sudo(parachainApi.tx.assetManager.setAssetUnitsPerSecond(assetId, 0))
  );
  return { events, assetId };
}

describeParachain(
  "XCM - receive_relay_asset_from_relay",
  { chain: "moonbase-local" },
  (context) => {
    it("should be able to receive an asset from relay", async function () {
      const keyring = new Keyring({ type: "sr25519" });
      const aliceRelay = keyring.addFromUri("//Alice");

      const alith = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");

      const parachainOne = context.polkadotApiParaone;
      const relayOne = context._polkadotApiRelaychains[0];

      // Log events
      logEvents(parachainOne, "PARA");

      logEvents(relayOne, "RELAY");

      await new Promise((res) => setTimeout(res, 10000));

      // PARACHAINS
      // registerAsset
      // const { events: eventsRegister } = await createBlockWithExtrinsicParachain(
      //   parachainOne,
      //   alith,
      //   parachainOne.tx.sudo.sudo(
      //     parachainOne.tx.assetManager.registerAsset(sourceLocation, relayAssetMetadata, new BN(1))
      //   )
      // );

      // // Look for assetId in events
      // let assetId: string;
      // eventsRegister.forEach((e) => {
      //   let ev = e.toHuman();
      //   if (ev.section === "assetManager") {
      //     assetId = ev.data[0];
      //   }
      // });
      // if (!assetId) {
      //   await new Promise((res) => setTimeout(res, 20000));
      // }
      // assetId = assetId.replace(/,/g, "");

      // setAssetUnitsPerSecond
      const { events, assetId } = await registerAssetToParachain(parachainOne, alith);
      // await createBlockWithExtrinsicParachain(
      //   parachainOne,
      //   alith,
      //   parachainOne.tx.sudo.sudo(parachainOne.tx.assetManager.setAssetUnitsPerSecond(assetId, 0))
      // );
      expect(events[0].toHuman().method).to.eq("UnitsPerSecondChanged");
      expect(events[2].toHuman().method).to.eq("ExtrinsicSuccess");

      // check asset in storage
      const registeredAsset = await parachainOne.query.assets.asset(assetId);
      expect((registeredAsset.toHuman() as { owner: string }).owner).to.eq(palletId);

      // RELAYCHAIN
      const { events: eventsRelay } = await createBlockWithExtrinsicParachain(
        relayOne,
        aliceRelay,
        relayOne.tx.xcmPallet.reserveTransferAssets(
          { X1: { Parachain: new BN(1000) } },
          { X1: { AccountKey20: { network: "Any", key: ALITH } } },
          [{ ConcreteFungible: { id: "Here", amount: new BN(1000000000000000) } }],
          new BN(4000000000)
        )
      );
      expect(eventsRelay[0].toHuman().method).to.eq("Attempted");

      // Wait for parachain block to have been emited
      await waitOneBlock(parachainOne, 2);

      // about 1k should have been substracted from AliceRelay
      expect(
        ((await relayOne.query.system.account(aliceRelay.address)) as any).data.free.toHuman()
      ).to.eq("8.9999 kUnit");
      // Alith asset balance should have been increased to 1000*e12
      expect(
        (await parachainOne.query.assets.account(assetId, ALITH)).toHuman().balance ===
          "1,000,000,000,000,000"
      ).to.eq(true);
    });
  }
);

describeParachain("XCM - send_relay_asset_to_relay", { chain: "moonbase-local" }, (context) => {
  let keyring: Keyring,
    aliceRelay: KeyringPair,
    alith: KeyringPair,
    baltathar: KeyringPair,
    parachainOne: ApiPromise,
    relayOne: ApiPromise,
    assetId: string;
  before("First send relay chain asset to parachain", async function () {
    keyring = new Keyring({ type: "sr25519" });

    // Setup Relaychain
    aliceRelay = keyring.addFromUri("//Alice");
    relayOne = context._polkadotApiRelaychains[0];

    // Setup Parachain
    alith = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
    baltathar = await keyring.addFromUri(BALTATHAR_PRIV_KEY, null, "ethereum");
    parachainOne = context.polkadotApiParaone;

    // Log events
    logEvents(parachainOne, "PARA");

    logEvents(relayOne, "RELAY");

    await new Promise((res) => setTimeout(res, 10000));

    // PARACHAIN A
    // registerAsset
    ({ assetId } = await registerAssetToParachain(parachainOne, alith));

    // RELAYCHAIN
    // Transfer 1000 units to para a baltathar
    await createBlockWithExtrinsicParachain(
      relayOne,
      aliceRelay,
      relayOne.tx.xcmPallet.reserveTransferAssets(
        { X1: { Parachain: new BN(1000) } },
        { X1: { AccountKey20: { network: "Any", key: BALTATHAR } } },
        [{ ConcreteFungible: { id: "Here", amount: new BN(THOUSAND_UNITS) } }],
        new BN(4000000000)
      )
    );

    // Wait for parachain block to have been emited
    await waitOneBlock(parachainOne, 2);

    // about 1k should have been substracted from AliceRelay (plus fees)
    expect(
      ((await relayOne.query.system.account(aliceRelay.address)) as any).data.free.toHuman()
    ).to.eq("8.9999 kUnit");
    // // Alith asset balance should have been increased to 1000*e12
    expect(
      (await parachainOne.query.assets.account(assetId, BALTATHAR)).toHuman().balance ===
        "1,000,000,000,000,000"
    ).to.eq(true);
  });
  it("should be able to receive an asset in relaychain from parachain", async function () {
    // PARACHAIN A
    // xToken transfer : sending 100 units back to relay
    const { events: eventsTransfer } = await createBlockWithExtrinsicParachain(
      parachainOne,
      baltathar,
      parachainOne.tx.xTokens.transfer(
        { OtherReserve: assetId },
        new BN(HUNDRED_UNITS),
        {
          X2: ["Parent", { AccountId32: { network: "Any", id: aliceRelay.addressRaw } }],
        },
        new BN(4000000000)
      )
    );
    expect(eventsTransfer[5].toHuman().method).to.eq("ExtrinsicSuccess");

    await waitOneBlock(relayOne, 3);
    // about 100 should have been added to AliceRelay (minus fees)
    expect(
      ((await relayOne.query.system.account(aliceRelay.address)) as any).data.free.toHuman()
    ).to.eq("9.0999 kUnit");
    // Baltathar should have 100 * 10^12 less
    expect(
      (await parachainOne.query.assets.account(assetId, BALTATHAR)).toHuman().balance ===
        "900,000,000,000,000"
    ).to.eq(true);
  });
});

describeParachain(
  "XCM - send_relay_asset_to_para_b - aka parachainTwo",
  { chain: "moonbase-local", numberOfParachains: 2 },
  (context) => {
    let keyring: Keyring,
      aliceRelay: KeyringPair,
      alith: KeyringPair,
      baltathar: KeyringPair,
      parachainOne: ApiPromise,
      parachainTwo: ApiPromise,
      relayOne: ApiPromise,
      assetId: string;
    before("First send relay chain asset to parachain", async function () {
      keyring = new Keyring({ type: "sr25519" });

      // Setup Relaychain
      aliceRelay = keyring.addFromUri("//Alice");
      relayOne = context._polkadotApiRelaychains[0];

      // Setup Parachains
      alith = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
      baltathar = await keyring.addFromUri(BALTATHAR_PRIV_KEY, null, "ethereum");
      parachainOne = context.polkadotApiParaone;
      parachainTwo = context._polkadotApiParachains[1][0];

      // Log events
      logEvents(parachainOne, "PARA A");
      logEvents(parachainTwo, "PARA B");
      logEvents(relayOne, "RELAY");

      await new Promise((res) => setTimeout(res, 2000));

      // PARACHAIN A
      // registerAsset
      ({ assetId } = await registerAssetToParachain(parachainOne, alith));

      // PARACHAIN B
      // registerAsset
      const { assetId: assetIdB } = await registerAssetToParachain(parachainTwo, alith);

      // They should have the same id
      expect(assetId).to.eq(assetIdB);

      // RELAYCHAIN
      // send 1000 units to Baltathar in para A
      const { events: eventsRelay } = await createBlockWithExtrinsicParachain(
        relayOne,
        aliceRelay,
        relayOne.tx.xcmPallet.reserveTransferAssets(
          { X1: { Parachain: new BN(1000) } },
          { X1: { AccountKey20: { network: "Any", key: BALTATHAR } } },
          [{ ConcreteFungible: { id: "Here", amount: new BN(THOUSAND_UNITS) } }],
          new BN(4000000000)
        )
      );
      // expect(eventsRelay[0].toHuman().method).to.eq("Attempted");

      // Wait for parachain block to have been emited
      await waitOneBlock(parachainOne, 2);

      expect(
        (await parachainOne.query.assets.account(assetId, BALTATHAR)).toHuman().balance ===
          "1,000,000,000,000,000"
      ).to.eq(true);
    });
    it.only("should be able to receive an asset in para b from para a", async function () {
      // PARACHAIN A
      // transfer 100 units to parachain B
      const { events: eventsTransfer } = await createBlockWithExtrinsicParachain(
        parachainOne,
        baltathar,
        parachainOne.tx.xTokens.transfer(
          { OtherReserve: assetId },
          new BN(HUNDRED_UNITS),
          {
            X3: [
              "Parent",
              { Parachain: new BN(2000) },
              { AccountKey20: { network: "Any", key: hexToU8a(BALTATHAR) } },
            ],
          },
          new BN(4000000000)
        )
      );
      await waitOneBlock(parachainTwo, 3);

      // about 1k should have been substracted from AliceRelay
      expect(
        ((await relayOne.query.system.account(aliceRelay.address)) as any).data.free.toHuman()
      ).to.eq("8.9999 kUnit");
      // Alith asset balance should have been increased to 1000*e12
      expect(
        (await parachainOne.query.assets.account(assetId, BALTATHAR)).toHuman().balance ===
          "900,000,000,000,000"
      ).to.eq(true);
      expect(
        (await parachainTwo.query.assets.account(assetId, BALTATHAR)).toHuman().balance ===
          "99,968,000,000,000"
      ).to.eq(true);
    });
  }
);

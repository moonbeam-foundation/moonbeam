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
const HUNDRED_UNITS_PARA = 100_000_000_000_000_000_000n;
const THOUSAND_UNITS = 1000000000000000;

interface AssetMetadata {
  name: string;
  symbol: string;
  decimals: BN;
  isFrozen: boolean;
}
const relayAssetMetadata: AssetMetadata = {
  name: "DOT",
  symbol: "DOT",
  decimals: new BN(12),
  isFrozen: false,
};
const paraAssetMetadata: AssetMetadata = {
  name: "GLMR",
  symbol: "GLMR",
  decimals: new BN(18),
  isFrozen: false,
};
const sourceLocationRelay = { XCM: { parents: 1, junctions: "Here" } }; // { XCM: "Parent" }; // { XCM: { X1: "Parent" } };

async function registerAssetToParachain(
  parachainApi: ApiPromise,
  sudoKeyring: KeyringPair,
  assetLocation: { XCM: any } = sourceLocationRelay,
  assetMetadata: AssetMetadata = relayAssetMetadata
) {
  const { events: eventsRegister } = await createBlockWithExtrinsicParachain(
    parachainApi,
    sudoKeyring,
    parachainApi.tx.sudo.sudo(
      parachainApi.tx.assetManager.registerAsset(assetLocation, assetMetadata, new BN(1))
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
  console.log("asset id", assetId);

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
    it.only("should be able to receive an asset from relay", async function () {
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
      const { events, assetId } = await registerAssetToParachain(parachainOne, alith);

      expect(events[0].toHuman().method).to.eq("UnitsPerSecondChanged");
      expect(events[2].toHuman().method).to.eq("ExtrinsicSuccess");

      // check asset in storage
      const registeredAsset = await parachainOne.query.assets.asset(assetId);
      expect((registeredAsset.toHuman() as { owner: string }).owner).to.eq(palletId);

      // RELAYCHAIN
      // Trigger the transfer
      const { events: eventsRelay } = await createBlockWithExtrinsicParachain(
        relayOne,
        aliceRelay,
        relayOne.tx.xcmPallet.reserveTransferAssets(
          { V1: { parents: new BN(0), interior: { X1: { Parachain: new BN(1000) } } } },

          {
            V1: {
              parents: new BN(0),
              interior: { X1: { AccountKey20: { network: "Any", key: ALITH } } },
            },
          },
          //[
          {
            V0: [{ ConcreteFungible: { id: "Here", amount: new BN(THOUSAND_UNITS) } }],
          },
          //], //["Here", new BN(1000000000000000)]],
          0,
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

      expect(
        (await parachainOne.query.assets.account(assetId, BALTATHAR)).toHuman().balance ===
          "1,000,000,000,000,000"
      ).to.eq(true);
    });
    it("should be able to receive a non-reserve asset in para b from para a", async function () {
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

describeParachain(
  "XCM - send_para_a_asset_to_para_b - aka parachainTwo",
  { chain: "moonbase-local", numberOfParachains: 2 },
  (context) => {
    let keyring: Keyring,
      alith: KeyringPair,
      baltathar: KeyringPair,
      parachainOne: ApiPromise,
      parachainTwo: ApiPromise,
      relayOne: ApiPromise,
      assetId: string,
      sourceLocationX3: { XCM: any },
      initialBalance: number;
    before("First send relay chain asset to parachain", async function () {
      keyring = new Keyring({ type: "ethereum" });

      // Setup Relaychain
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

      initialBalance = Number((await parachainOne.query.system.account(BALTATHAR)).data.free);

      // Get Pallet balances index
      const metadata = await parachainOne.rpc.state.getMetadata();
      const palletIndex = (metadata.asLatest.toHuman().modules as Array<any>).find((pallet) => {
        return pallet.name === "Balances";
      }).index;

      expect(palletIndex);

      sourceLocationX3 = {
        XCM: {
          X3: ["Parent", { Parachain: new BN(1000) }, { Palletinstance: new BN(palletIndex) }],
        },
      };

      // PARACHAIN B
      // registerAsset
      ({ assetId } = await registerAssetToParachain(
        parachainTwo,
        alith,
        sourceLocationX3,
        paraAssetMetadata
      ));
    });
    it("should be able to receive an asset in para b from para a", async function () {
      // PARACHAIN A
      // transfer 100 units to parachain B
      const { events: eventsTransfer } = await createBlockWithExtrinsicParachain(
        parachainOne,
        baltathar,
        parachainOne.tx.xTokens.transfer(
          "SelfReserve",
          HUNDRED_UNITS_PARA,
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

      expect(eventsTransfer[2].toHuman().method).to.eq("XcmpMessageSent");
      expect(eventsTransfer[3].toHuman().method).to.eq("Transferred");
      expect(eventsTransfer[7].toHuman().method).to.eq("ExtrinsicSuccess");

      await waitOneBlock(parachainTwo, 3);

      // Verify that difference is 100 units plus fees (less than 1% of 10^18)
      const targetBalance: number = Number(BigInt(BigInt(initialBalance) - HUNDRED_UNITS_PARA));
      const diff =
        Number((await parachainOne.query.system.account(BALTATHAR)).data.free) - targetBalance;
      expect(diff < 10000000000000000).to.eq(true);
      expect(
        (await parachainTwo.query.assets.account(assetId, BALTATHAR)).toHuman().balance ===
          "100,000,000,000,000,000,000"
      ).to.eq(true);
    });
  }
);

describeParachain(
  "XCM - send_para_a_asset_to_para_b_and_back_to_para_a - aka parachainTwo",
  { chain: "moonbase-local", numberOfParachains: 2 },
  (context) => {
    let keyring: Keyring,
      alith: KeyringPair,
      baltathar: KeyringPair,
      parachainOne: ApiPromise,
      parachainTwo: ApiPromise,
      relayOne: ApiPromise,
      assetId: string,
      sourceLocationX3: { XCM: any },
      initialBalance: number;
    before("First send relay chain asset to parachain", async function () {
      keyring = new Keyring({ type: "ethereum" });

      // Setup Relaychain
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

      initialBalance = Number((await parachainOne.query.system.account(BALTATHAR)).data.free);

      // Get Pallet balances index
      const metadata = await parachainOne.rpc.state.getMetadata();
      const palletIndex = (metadata.asLatest.toHuman().modules as Array<any>).find((pallet) => {
        return pallet.name === "Balances";
      }).index;

      expect(palletIndex);

      sourceLocationX3 = {
        XCM: {
          X3: ["Parent", { Parachain: new BN(1000) }, { Palletinstance: new BN(palletIndex) }],
        },
      };

      // PARACHAIN B
      // registerAsset
      ({ assetId } = await registerAssetToParachain(
        parachainTwo,
        alith,
        sourceLocationX3,
        paraAssetMetadata
      ));

      // PARACHAIN A
      // transfer 100 units to parachain B
      await createBlockWithExtrinsicParachain(
        parachainOne,
        baltathar,
        parachainOne.tx.xTokens.transfer(
          "SelfReserve",
          HUNDRED_UNITS_PARA,
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
    });
    it("should be able to receive an asset in para b from para a", async function () {
      // PARACHAIN B
      // transfer back 100 units to parachain A
      const { events: eventsTransfer } = await createBlockWithExtrinsicParachain(
        parachainTwo,
        baltathar,
        parachainTwo.tx.xTokens.transfer(
          { OtherReserve: assetId },
          HUNDRED_UNITS_PARA,
          {
            X3: [
              "Parent",
              { Parachain: new BN(1000) },
              { AccountKey20: { network: "Any", key: hexToU8a(BALTATHAR) } },
            ],
          },
          new BN(4000000000)
        )
      );
      expect(eventsTransfer[1].toHuman().method).to.eq("XcmpMessageSent");
      expect(eventsTransfer[2].toHuman().method).to.eq("Transferred");
      expect(eventsTransfer[6].toHuman().method).to.eq("ExtrinsicSuccess");

      await waitOneBlock(parachainTwo, 3);

      const diff =
        initialBalance - Number((await parachainOne.query.system.account(BALTATHAR)).data.free);
      // Verify that difference is fees (less than 1% of 10^18)
      expect(diff < 10000000000000000).to.eq(true);
      expect((await parachainTwo.query.assets.account(assetId, BALTATHAR)).toHuman().balance).to.eq(
        "0"
      );
    });
  }
);

describeParachain(
  "XCM - send_para_a_asset_from_para_b_to_para_c",
  { chain: "moonbase-local", numberOfParachains: 3 },
  (context) => {
    let keyring: Keyring,
      alith: KeyringPair,
      baltathar: KeyringPair,
      parachainOne: ApiPromise,
      parachainTwo: ApiPromise,
      parachainThree: ApiPromise,
      relayOne: ApiPromise,
      assetId: string,
      sourceLocationX3: { XCM: any },
      initialBalance: number;
    before("First send relay chain asset to parachain", async function () {
      keyring = new Keyring({ type: "ethereum" });

      // Setup Relaychain
      relayOne = context._polkadotApiRelaychains[0];

      // Setup Parachains
      alith = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
      baltathar = await keyring.addFromUri(BALTATHAR_PRIV_KEY, null, "ethereum");
      parachainOne = context.polkadotApiParaone;
      parachainTwo = context._polkadotApiParachains[1][0];
      parachainThree = context._polkadotApiParachains[2][0];

      // Log events
      logEvents(parachainOne, "PARA A");
      logEvents(parachainTwo, "PARA B");
      logEvents(parachainThree, "PARA C");
      logEvents(relayOne, "RELAY");

      initialBalance = Number((await parachainOne.query.system.account(BALTATHAR)).data.free);

      // Get Pallet balances index
      const metadata = await parachainOne.rpc.state.getMetadata();
      const palletIndex = (metadata.asLatest.toHuman().modules as Array<any>).find((pallet) => {
        return pallet.name === "Balances";
      }).index;

      expect(palletIndex);

      sourceLocationX3 = {
        XCM: {
          X3: ["Parent", { Parachain: new BN(1000) }, { Palletinstance: new BN(palletIndex) }],
        },
      };

      // PARACHAIN B
      // registerAsset
      ({ assetId } = await registerAssetToParachain(
        parachainTwo,
        alith,
        sourceLocationX3,
        paraAssetMetadata
      ));

      // PARACHAIN C
      // registerAsset
      await registerAssetToParachain(parachainThree, alith, sourceLocationX3, paraAssetMetadata);
    });
    it("should be able to receive an asset back in para a from para b", async function () {
      // PARACHAIN A
      // transfer 100 units to parachain B
      await createBlockWithExtrinsicParachain(
        parachainOne,
        baltathar,
        parachainOne.tx.xTokens.transfer(
          "SelfReserve",
          HUNDRED_UNITS_PARA,
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

      // PARACHAIN B
      // transfer 100 units to parachain C
      const { events: eventsTransfer2 } = await createBlockWithExtrinsicParachain(
        parachainTwo,
        baltathar,
        parachainTwo.tx.xTokens.transfer(
          { OtherReserve: assetId },
          HUNDRED_UNITS_PARA,
          {
            X3: [
              "Parent",
              { Parachain: new BN(3000) },
              { AccountKey20: { network: "Any", key: hexToU8a(BALTATHAR) } },
            ],
          },
          new BN(4000000000)
        )
      );

      expect(eventsTransfer2[1].toHuman().method).to.eq("XcmpMessageSent");
      expect(eventsTransfer2[2].toHuman().method).to.eq("Transferred");
      expect(eventsTransfer2[6].toHuman().method).to.eq("ExtrinsicSuccess");

      await waitOneBlock(parachainThree, 6);
      // Verify that difference is 100 units plus fees (less than 1% of 10^18)
      const targetBalance: number = Number(BigInt(BigInt(initialBalance) - HUNDRED_UNITS_PARA));
      const diff =
        Number((await parachainOne.query.system.account(BALTATHAR)).data.free) - targetBalance;
      expect(diff < 10000000000000000).to.eq(true);
      expect((await parachainTwo.query.assets.account(assetId, BALTATHAR)).toHuman().balance).to.eq(
        "0"
      );
      expect(
        (await parachainThree.query.assets.account(assetId, BALTATHAR)).toHuman().balance
      ).to.eq("99,999,999,996,000,000,000");
    });
  }
);

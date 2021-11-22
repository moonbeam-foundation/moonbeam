import Keyring from "@polkadot/keyring";
import { expect } from "chai";
import { BN, hexToU8a } from "@polkadot/util";

import { ALITH, ALITH_PRIV_KEY, BALTATHAR, BALTATHAR_PRIV_KEY } from "../util/constants";
import { describeParachain } from "../util/setup-para-tests";
import { createBlockWithExtrinsicParachain, logEvents, waitOneBlock } from "../util/substrate-rpc";
import { KeyringPair } from "@polkadot/keyring/types";
import { ApiPromise } from "@polkadot/api";
import { BigNumber } from "@ethersproject/bignumber";

const palletId = "0x6d6f646c617373746d6e67720000000000000000";
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
interface SourceLocation {
  XCM: {
    parents: number | BN;
    interior: any;
  };
}
const sourceLocationRelay = { XCM: { parents: 1, interior: "Here" } };

async function registerAssetToParachain(
  parachainApi: ApiPromise,
  sudoKeyring: KeyringPair,
  assetLocation: SourceLocation = sourceLocationRelay,
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
      const { events, assetId } = await registerAssetToParachain(parachainOne, alith);

      expect(events[1].toHuman().method).to.eq("UnitsPerSecondChanged");
      expect(events[4].toHuman().method).to.eq("ExtrinsicSuccess");

      // check asset in storage
      const registeredAsset = await parachainOne.query.assets.asset(assetId);
      expect((registeredAsset.toHuman() as { owner: string }).owner).to.eq(palletId);

      // RELAYCHAIN

      // Set supported version
      // Release-v0.9.12 does not have yet automatic versioning
      await createBlockWithExtrinsicParachain(
        relayOne,
        aliceRelay,
        relayOne.tx.sudo.sudo(relayOne.tx.xcmPallet.forceDefaultXcmVersion(1))
      );

      let beforeAliceRelayBalance = (
        (await relayOne.query.system.account(aliceRelay.address)) as any
      ).data.free;
      // Fees
      const fee = (
        await relayOne.tx.xcmPallet
          .reserveTransferAssets(
            { V1: { parents: new BN(0), interior: { X1: { Parachain: new BN(1000) } } } },
            {
              V1: {
                parents: new BN(0),
                interior: { X1: { AccountKey20: { network: "Any", key: BALTATHAR } } },
              },
            },
            {
              V0: [{ ConcreteFungible: { id: "Null", amount: new BN(THOUSAND_UNITS) } }],
            },
            0
          )
          .paymentInfo(aliceRelay)
      ).partialFee as any;
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
          {
            V0: [{ ConcreteFungible: { id: "Null", amount: new BN(THOUSAND_UNITS) } }],
          },
          0
        )
      );

      let expectedAfterRelayBalance =
        BigInt(beforeAliceRelayBalance) - BigInt(fee) - BigInt(THOUSAND_UNITS);
      expect(eventsRelay[3].toHuman().method).to.eq("Attempted");

      // Wait for parachain block to have been emited
      await waitOneBlock(parachainOne, 2);
      // about 1k should have been substracted from AliceRelay
      let afterAliceRelayBalance = (
        (await relayOne.query.system.account(aliceRelay.address)) as any
      ).data.free;

      expect(afterAliceRelayBalance.toString()).to.eq(expectedAfterRelayBalance.toString());

      // Alith asset balance should have been increased to 1000*e12
      expect(
        ((await parachainOne.query.assets.account(assetId, ALITH)).balance as any).toString()
      ).to.eq(BigInt(THOUSAND_UNITS).toString());
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
  before("First send rela,y chain asset to parachain", async function () {
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
    // Set supported version
    // Release-v0.9.12 does not have yet automatic versioning
    await createBlockWithExtrinsicParachain(
      relayOne,
      aliceRelay,
      relayOne.tx.sudo.sudo(relayOne.tx.xcmPallet.forceDefaultXcmVersion(1))
    );

    let beforeAliceRelayBalance = ((await relayOne.query.system.account(aliceRelay.address)) as any)
      .data.free;
    // Fees
    const fee = (
      await relayOne.tx.xcmPallet
        .reserveTransferAssets(
          { V1: { parents: new BN(0), interior: { X1: { Parachain: new BN(1000) } } } },
          {
            V1: {
              parents: new BN(0),
              interior: { X1: { AccountKey20: { network: "Any", key: BALTATHAR } } },
            },
          },
          {
            V0: [{ ConcreteFungible: { id: "Null", amount: new BN(THOUSAND_UNITS) } }],
          },
          0
        )
        .paymentInfo(aliceRelay)
    ).partialFee as any;

    let expectedAfterRelayBalance =
      BigInt(beforeAliceRelayBalance) - BigInt(fee) - BigInt(THOUSAND_UNITS);

    // Transfer 1000 units to para a baltathar
    await createBlockWithExtrinsicParachain(
      relayOne,
      aliceRelay,
      relayOne.tx.xcmPallet.reserveTransferAssets(
        { V1: { parents: new BN(0), interior: { X1: { Parachain: new BN(1000) } } } },
        {
          V1: {
            parents: new BN(0),
            interior: { X1: { AccountKey20: { network: "Any", key: BALTATHAR } } },
          },
        },
        {
          V0: [{ ConcreteFungible: { id: "Null", amount: new BN(THOUSAND_UNITS) } }],
        },
        0
      )
    );

    // Wait for parachain block to have been emited
    await waitOneBlock(parachainOne, 2);

    // about 1k should have been substracted from AliceRelay (plus fees)
    let afterAliceRelayBalance = ((await relayOne.query.system.account(aliceRelay.address)) as any)
      .data.free;

    expect(afterAliceRelayBalance.toString()).to.eq(expectedAfterRelayBalance.toString());
    // // BALTATHAR asset balance should have been increased to 1000*e12
    expect(
      ((await parachainOne.query.assets.account(assetId, BALTATHAR)).balance as any).toString()
    ).to.eq(BigInt(THOUSAND_UNITS).toString());
  });
  it("should be able to receive an asset in relaychain from parachain", async function () {
    // about 1k should have been substracted from AliceRelay (plus fees)
    let beforeAliceRelayBalance = ((await relayOne.query.system.account(aliceRelay.address)) as any)
      .data.free;
    // PARACHAIN A
    // xToken transfer : sending 100 units back to relay
    const { events: eventsTransfer } = await createBlockWithExtrinsicParachain(
      parachainOne,
      baltathar,
      parachainOne.tx.xTokens.transfer(
        { OtherReserve: assetId },
        new BN(HUNDRED_UNITS),
        {
          V1: {
            parents: new BN(1),
            interior: { X1: { AccountId32: { network: "Any", id: aliceRelay.addressRaw } } },
          },
        },
        new BN(4000000000)
      )
    );

    expect(eventsTransfer[7].toHuman().method).to.eq("ExtrinsicSuccess");
    // Constant, but we cannot easily take them
    // Fees related to WithdrawAsset + ClearOrigin+ BuyExecution + DepositAsset
    // I think this corresponds to 8 units per weight times 1000000000 pero instruction
    let fees = BigInt(32000000000);

    let expectedAliceBalance = BigInt(beforeAliceRelayBalance) + BigInt(HUNDRED_UNITS) - fees;

    await waitOneBlock(relayOne, 3);
    // about 100 should have been added to AliceRelay (minus fees)
    expect(
      ((await relayOne.query.system.account(aliceRelay.address)) as any).data.free.toString()
    ).to.eq(expectedAliceBalance.toString());
    // Baltathar should have 100 * 10^12 less
    expect(
      ((await parachainOne.query.assets.account(assetId, BALTATHAR)).balance as any).toString()
    ).to.eq((BigInt(THOUSAND_UNITS) - BigInt(HUNDRED_UNITS)).toString());
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
      parachainTwo = context._polkadotApiParachains[1].apis[0];

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
      // Set supported version
      // Release-v0.9.12 does not have yet automatic versioning
      await createBlockWithExtrinsicParachain(
        relayOne,
        aliceRelay,
        relayOne.tx.sudo.sudo(relayOne.tx.xcmPallet.forceDefaultXcmVersion(1))
      );

      let beforeAliceRelayBalance = (
        (await relayOne.query.system.account(aliceRelay.address)) as any
      ).data.free;
      // Fees
      const fee = (
        await relayOne.tx.xcmPallet
          .reserveTransferAssets(
            { V1: { parents: new BN(0), interior: { X1: { Parachain: new BN(1000) } } } },
            {
              V1: {
                parents: new BN(0),
                interior: { X1: { AccountKey20: { network: "Any", key: BALTATHAR } } },
              },
            },
            {
              V0: [{ ConcreteFungible: { id: "Null", amount: new BN(THOUSAND_UNITS) } }],
            },
            0
          )
          .paymentInfo(aliceRelay)
      ).partialFee as any;

      let expectedAfterRelayBalance =
        BigInt(beforeAliceRelayBalance) - BigInt(fee) - BigInt(THOUSAND_UNITS);

      // Transfer 1000 units to para a baltathar
      await createBlockWithExtrinsicParachain(
        relayOne,
        aliceRelay,
        relayOne.tx.xcmPallet.reserveTransferAssets(
          { V1: { parents: new BN(0), interior: { X1: { Parachain: new BN(1000) } } } },
          {
            V1: {
              parents: new BN(0),
              interior: { X1: { AccountKey20: { network: "Any", key: BALTATHAR } } },
            },
          },
          {
            V0: [{ ConcreteFungible: { id: "Null", amount: new BN(THOUSAND_UNITS) } }],
          },
          0
        )
      );

      // Wait for parachain block to have been emited
      await waitOneBlock(parachainOne, 2);

      let afterAliceRelayBalance = (
        (await relayOne.query.system.account(aliceRelay.address)) as any
      ).data.free;

      expect(afterAliceRelayBalance.toString()).to.eq(expectedAfterRelayBalance.toString());

      expect(
        ((await parachainOne.query.assets.account(assetId, BALTATHAR)).balance as any).toString()
      ).to.eq(BigInt(THOUSAND_UNITS).toString());
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
            V1: {
              parents: new BN(1),
              interior: {
                X2: [
                  { Parachain: new BN(2000) },
                  { AccountKey20: { network: "Any", key: hexToU8a(BALTATHAR) } },
                ],
              },
            },
          },
          new BN(4000000000)
        )
      );

      await waitOneBlock(parachainTwo, 3);

      // These are related to the operations in the relay
      // Constant, but we cannot easily take them
      // Fees related to WithdrawAsset + ClearOrigin + DepositReserveAsset + ReserveAssetDeposited
      // I think this corresponds to 8 units per weight times 1000000000 pero instruction
      let relayFees = BigInt(32000000000);
      let expectedBaltatharParaTwoBalance = BigInt(HUNDRED_UNITS) - relayFees;

      expect(
        ((await parachainOne.query.assets.account(assetId, BALTATHAR)).balance as any).toString()
      ).to.eq((BigInt(THOUSAND_UNITS) - BigInt(HUNDRED_UNITS)).toString());
      expect(
        ((await parachainTwo.query.assets.account(assetId, BALTATHAR)).balance as any).toString()
      ).to.eq(expectedBaltatharParaTwoBalance.toString());
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
      sourceLocationParaA: SourceLocation,
      initialBalance: number;
    before("First send relay chain asset to parachain", async function () {
      keyring = new Keyring({ type: "ethereum" });

      // Setup Relaychain
      relayOne = context._polkadotApiRelaychains[0];

      // Setup Parachains
      alith = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
      baltathar = await keyring.addFromUri(BALTATHAR_PRIV_KEY, null, "ethereum");
      parachainOne = context.polkadotApiParaone;
      parachainTwo = context._polkadotApiParachains[1].apis[0];

      // Log events
      logEvents(parachainOne, "PARA A");
      logEvents(parachainTwo, "PARA B");
      logEvents(relayOne, "RELAY");

      initialBalance = Number((await parachainOne.query.system.account(BALTATHAR)).data.free);

      // Get Pallet balances index
      const metadata = await parachainOne.rpc.state.getMetadata();
      const palletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find((pallet) => {
        return pallet.name === "Balances";
      }).index;

      expect(palletIndex);

      sourceLocationParaA = {
        XCM: {
          parents: 1,
          interior: { X2: [{ Parachain: new BN(1000) }, { Palletinstance: new BN(palletIndex) }] },
        },
      };

      // PARACHAIN B
      // registerAsset
      ({ assetId } = await registerAssetToParachain(
        parachainTwo,
        alith,
        sourceLocationParaA,
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
            V1: {
              parents: new BN(1),
              interior: {
                X2: [
                  { Parachain: new BN(2000) },
                  { AccountKey20: { network: "Any", key: hexToU8a(BALTATHAR) } },
                ],
              },
            },
          },
          new BN(4000000000)
        )
      );

      expect(eventsTransfer[5].toHuman().method).to.eq("XcmpMessageSent");
      expect(eventsTransfer[6].toHuman().method).to.eq("Transferred");
      expect(eventsTransfer[11].toHuman().method).to.eq("ExtrinsicSuccess");

      await waitOneBlock(parachainTwo, 3);

      // Verify that difference is 100 units plus fees (less than 1% of 10^18)
      const targetBalance: number = Number(BigInt(BigInt(initialBalance) - HUNDRED_UNITS_PARA));
      const diff =
        Number((await parachainOne.query.system.account(BALTATHAR)).data.free) - targetBalance;
      expect(diff < 10000000000000000).to.eq(true);

      let expectedBaltatharParaTwoBalance = BigInt(HUNDRED_UNITS_PARA);

      expect(
        ((await parachainTwo.query.assets.account(assetId, BALTATHAR)).balance as any).toString()
      ).to.eq(expectedBaltatharParaTwoBalance.toString());
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
      sourceLocationParaA: SourceLocation,
      initialBalance: number;
    before("First send relay chain asset to parachain", async function () {
      keyring = new Keyring({ type: "ethereum" });

      // Setup Relaychain
      relayOne = context._polkadotApiRelaychains[0];

      // Setup Parachains
      alith = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
      baltathar = await keyring.addFromUri(BALTATHAR_PRIV_KEY, null, "ethereum");
      parachainOne = context.polkadotApiParaone;
      parachainTwo = context._polkadotApiParachains[1].apis[0];

      // Log events
      logEvents(parachainOne, "PARA A");
      logEvents(parachainTwo, "PARA B");
      logEvents(relayOne, "RELAY");

      initialBalance = Number((await parachainOne.query.system.account(BALTATHAR)).data.free);

      // Get Pallet balances index
      const metadata = await parachainOne.rpc.state.getMetadata();
      const palletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find((pallet) => {
        return pallet.name === "Balances";
      }).index;

      expect(palletIndex);

      sourceLocationParaA = {
        XCM: {
          parents: new BN(1),
          interior: { X2: [{ Parachain: new BN(1000) }, { Palletinstance: new BN(palletIndex) }] },
        },
      };

      // PARACHAIN B
      // registerAsset
      ({ assetId } = await registerAssetToParachain(
        parachainTwo,
        alith,
        sourceLocationParaA,
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
            V1: {
              parents: new BN(1),
              interior: {
                X2: [
                  { Parachain: new BN(2000) },
                  { AccountKey20: { network: "Any", key: hexToU8a(BALTATHAR) } },
                ],
              },
            },
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
            V1: {
              parents: new BN(1),
              interior: {
                X2: [
                  { Parachain: new BN(1000) },
                  { AccountKey20: { network: "Any", key: hexToU8a(BALTATHAR) } },
                ],
              },
            },
          },
          new BN(4000000000)
        )
      );

      expect(eventsTransfer[2].toHuman().method).to.eq("XcmpMessageSent");
      expect(eventsTransfer[3].toHuman().method).to.eq("Transferred");
      expect(eventsTransfer[8].toHuman().method).to.eq("ExtrinsicSuccess");

      await waitOneBlock(parachainTwo, 3);

      const diff =
        initialBalance - Number((await parachainOne.query.system.account(BALTATHAR)).data.free);
      // Verify that difference is fees (less than 1% of 10^18)
      expect(diff < 10000000000000000).to.eq(true);

      let expectedBaltatharParaTwoBalance = BigInt(0);

      expect(
        ((await parachainTwo.query.assets.account(assetId, BALTATHAR)).balance as any).toString()
      ).to.eq(expectedBaltatharParaTwoBalance.toString());
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
      sourceLocationParaA: SourceLocation,
      initialBalance: number;
    before("First send relay chain asset to parachain", async function () {
      keyring = new Keyring({ type: "ethereum" });

      // Setup Relaychain
      relayOne = context._polkadotApiRelaychains[0];

      // Setup Parachains
      alith = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
      baltathar = await keyring.addFromUri(BALTATHAR_PRIV_KEY, null, "ethereum");
      parachainOne = context.polkadotApiParaone;
      parachainTwo = context._polkadotApiParachains[1].apis[0];
      parachainThree = context._polkadotApiParachains[2].apis[0];

      // Log events
      logEvents(parachainOne, "PARA A");
      logEvents(parachainTwo, "PARA B");
      logEvents(parachainThree, "PARA C");
      logEvents(relayOne, "RELAY");

      initialBalance = Number((await parachainOne.query.system.account(BALTATHAR)).data.free);

      // Get Pallet balances index
      const metadata = await parachainOne.rpc.state.getMetadata();
      const palletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find((pallet) => {
        return pallet.name === "Balances";
      }).index;

      expect(palletIndex);

      sourceLocationParaA = {
        XCM: {
          parents: new BN(1),
          interior: { X2: [{ Parachain: new BN(1000) }, { Palletinstance: new BN(palletIndex) }] },
        },
      };

      // PARACHAIN B
      // registerAsset
      ({ assetId } = await registerAssetToParachain(
        parachainTwo,
        alith,
        sourceLocationParaA,
        paraAssetMetadata
      ));

      // PARACHAIN C
      // registerAsset
      await registerAssetToParachain(parachainThree, alith, sourceLocationParaA, paraAssetMetadata);
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
            V1: {
              parents: new BN(1),
              interior: {
                X2: [
                  { Parachain: new BN(2000) },
                  { AccountKey20: { network: "Any", key: hexToU8a(BALTATHAR) } },
                ],
              },
            },
          },
          new BN(4000000000)
        )
      );

      await waitOneBlock(parachainTwo, 6);

      // PARACHAIN B
      // transfer 100 units to parachain C
      const { events: eventsTransfer2 } = await createBlockWithExtrinsicParachain(
        parachainTwo,
        baltathar,
        parachainTwo.tx.xTokens.transfer(
          { OtherReserve: assetId },
          HUNDRED_UNITS_PARA,
          {
            V1: {
              parents: new BN(1),
              interior: {
                X2: [
                  { Parachain: new BN(3000) },
                  { AccountKey20: { network: "Any", key: hexToU8a(BALTATHAR) } },
                ],
              },
            },
          },
          new BN(4000000000)
        )
      );

      expect(eventsTransfer2[2].toHuman().method).to.eq("XcmpMessageSent");
      expect(eventsTransfer2[3].toHuman().method).to.eq("Transferred");
      expect(eventsTransfer2[8].toHuman().method).to.eq("ExtrinsicSuccess");

      await waitOneBlock(parachainThree, 3);
      // Verify that difference is 100 units plus fees (less than 1% of 10^18)
      const targetBalance: number = Number(BigInt(BigInt(initialBalance) - HUNDRED_UNITS_PARA));
      let expectedBaltatharParaTwoBalance = BigInt(0);
      let paraAXcmFee = BigInt(400000000);
      const diff =
        Number((await parachainOne.query.system.account(BALTATHAR)).data.free) - targetBalance;
      expect(diff < 10000000000000000).to.eq(true);
      expect(
        ((await parachainTwo.query.assets.account(assetId, BALTATHAR)).balance as any).toString()
      ).to.eq(expectedBaltatharParaTwoBalance.toString());
      expect(
        (await parachainThree.query.assets.account(assetId, BALTATHAR)).balance.toString()
      ).to.eq((BigInt(HUNDRED_UNITS_PARA) - paraAXcmFee).toString());
    });
  }
);

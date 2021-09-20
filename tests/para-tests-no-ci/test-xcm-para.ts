import Keyring from "@polkadot/keyring";
import { expect } from "chai";
import { BN } from "@polkadot/util";

import { ALITH, ALITH_PRIV_KEY } from "../util/constants";
import { describeParachain } from "../util/setup-para-tests";
import { createBlockWithExtrinsicParachain, logEvents, waitOneBlock } from "../util/substrate-rpc";

const palletId = "0x6D6f646c617373746d6E67720000000000000000";

const assetMetadata = {
  name: "DOT",
  symbol: "DOT",
  decimals: new BN(12),
  isFrozen: false,
};
const sourceLocation = { XCM: { X1: "Parent" } };

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
      const { events: eventsRegister } = await createBlockWithExtrinsicParachain(
        parachainOne,
        alith,
        parachainOne.tx.sudo.sudo(
          parachainOne.tx.assetManager.registerAsset(sourceLocation, assetMetadata, new BN(1))
        )
      );

      // Look for assetId in events
      let assetId: string;
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
      console.log("assetId", assetId);

      // setAssetUnitsPerSecond
      const { events } = await createBlockWithExtrinsicParachain(
        parachainOne,
        alith,
        parachainOne.tx.sudo.sudo(parachainOne.tx.assetManager.setAssetUnitsPerSecond(assetId, 0))
      );
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
      console.log((await parachainOne.query.assets.account(assetId, ALITH)).toHuman().balance);
      expect(
        (await parachainOne.query.assets.account(assetId, ALITH)).toHuman().balance ===
          "1,000,000,000,000,000"
      ).to.eq(true);
    });
  }
);

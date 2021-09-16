import Keyring from "@polkadot/keyring";
import { blake2AsHex } from "@polkadot/util-crypto";
import { expect } from "chai";
import { BN, isUndefined } from "@polkadot/util";

import {
  ALITH,
  ALITH_PRIV_KEY,
  GENESIS_ACCOUNT,
  GENESIS_ACCOUNT_BALANCE,
  GENESIS_ACCOUNT_PRIVATE_KEY,
} from "../../util/constants";
import { describeParachain } from "../../util/setup-para-tests";
import {
  createBlockWithExtrinsic,
  createBlockWithExtrinsicParachain,
} from "../../util/substrate-rpc";

const palletId = "0x6D6f646c617373746d6E67720000000000000000";

const assetMetadata = {
  name: "DOT",
  symbol: "DOT",
  decimals: new BN(12),
  isFrozen: false,
};
const sourceLocation = { XCM: { interior: { Here: null }, parents: new BN(1) } }; //{ XCM: { X1: "Parent" } };
// const sourceId = blake2AsHex(JSON.stringify(sourceLocation));

describeParachain(
  "XCM - receive_relay_asset_from_relay",
  { chain: "moonbase-local" },
  (context) => {
    it("should be accessible through web3", async function () {
      const keyring = new Keyring({ type: "sr25519" });
      const aliceRelay = keyring.addFromUri("//Alice");

      const alith = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
      const genesisAccount = await keyring.addFromUri(
        GENESIS_ACCOUNT_PRIVATE_KEY,
        null,
        "ethereum"
      );

      const parachainOne = context.polkadotApiParaone;
      const relayOne = context._polkadotApiRelaychains[0];

      // subscribe to all new headers (with extended info)
      context.polkadotApiParaone.derive.chain.subscribeNewHeads((header) => {
        console.log(`#${header.number}: ${header.author}`);
      });
      await new Promise((res) => setTimeout(res, 20000));

      console.log("before", (await parachainOne.query.system.account(ALITH)).data.free.toHuman());

      // PARACHAINS
      // registerAsset
      const { events: eventsRegister } = await createBlockWithExtrinsicParachain(
        context,
        alith,
        parachainOne.tx.sudo.sudo(
          parachainOne.tx.assetManager.registerAsset(sourceLocation, assetMetadata, new BN(1))
        )
      );
      // Look for assetId in events
      let assetId: string;
      eventsRegister.forEach((e) => {
        console.log(e.toHuman());
        let ev = e.toHuman();
        if (ev.section === "assetManager") {
          assetId = ev.data[0];
        }
      });
      assetId = assetId.replace(/,/g, "");

      // setAssetUnitsPerSecond
      const { events } = await createBlockWithExtrinsicParachain(
        context,
        alith,
        parachainOne.tx.sudo.sudo(parachainOne.tx.assetManager.setAssetUnitsPerSecond(assetId, 0))
      );
      events.forEach((e) => console.log(e.toHuman()));
      expect(events[0].toHuman().method).to.eq("UnitsPerSecondChanged");
      expect(events[2].toHuman().method).to.eq("ExtrinsicSuccess");

      // check asset in storage
      const registeredAsset = await parachainOne.query.assets.asset(assetId);
      expect((registeredAsset.toHuman() as { owner: string }).owner).to.eq(palletId);

      // RELAYCHAIN
      const res3 = await relayOne.tx.xcmPallet
        .reserveTransferAssets(
          { X1: { Parachain: new BN(1000) } },
          { X1: { network: "Any", key: ALITH } },
          [{ id: "Here", amount: new BN(1000000000000000) }],
          new BN(4000000000)
        )
        .signAndSend(aliceRelay); // NO SUDO FOR RELAY
      console.log("res3", res3);
      console.log("after", (await parachainOne.query.system.account(ALITH)).data.free.toHuman());
      expect((await parachainOne.query.system.account(ALITH)).data.free.toHuman()).to.eq(
        "1.2078 MMOVR"
      );
    });
  }
);

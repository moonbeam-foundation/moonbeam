import Keyring from "@polkadot/keyring";
import { blake2AsHex } from "@polkadot/util-crypto";
import { expect } from "chai";

import {
  ALITH,
  ALITH_PRIV_KEY,
  GENESIS_ACCOUNT,
  GENESIS_ACCOUNT_BALANCE,
  GENESIS_ACCOUNT_PRIVATE_KEY,
} from "../util/constants";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { describeParachain } from "../util/setup-para-tests";

const MOONRIVER_SUDO_ACCOUNT = "0xb728c13034c3b6c6447f399d25b097216a0081ea";

const assetMetadata = {
  name: "DOT",
  symbol: "DOT",
  decimals: 12,
};
const sourceLocation = { XCM: { X1: "Parent" } };
const sourceId = blake2AsHex(JSON.stringify(sourceLocation));

describeDevMoonbeam(
  "XCM - receive_relay_asset_from_relay",
  //{ chain: "moonriver-local" },
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

      const parachainOne = context.polkadotApi;
      // const relayOne = context._polkadotApiRelaychains[0];

      console.log("before", (await parachainOne.query.system.account(ALITH)).data.free.toHuman());
      console.log(
        "before genesis",
        (await parachainOne.query.system.account(GENESIS_ACCOUNT)).data.free.toHuman()
      );

      // parachains
      console.log(Object.keys(parachainOne.tx));
      const res = await parachainOne.tx.sudo
        .sudo(parachainOne.tx.assetManager.registerAsset(sourceLocation, assetMetadata, 1))
        .signAndSend(genesisAccount);
      console.log("res", res);

      const res2 = await parachainOne.tx.sudo
        .sudo(parachainOne.tx.assetManager.setUnitsPerSecond(sourceId, 0))
        .signAndSend(genesisAccount);

      console.log("res2", res2);

      const res3 = await parachainOne.query.assetManager.assetIdType(null);
      const res4 = await parachainOne.query.assetManager.assetIdType(sourceId);
      console.log(res3.toHuman());
      console.log(res4.toHuman());

      //relay
      // const res3 = await relayOne.tx.xcmPallet
      //   .reserveTransferAssets(
      //     { X1: { Parachain: 1000 } },
      //     { X1: { network: "Any", key: ALITH } },
      //     [{ id: "Here", amount: 1000000000000000 }],
      //     4000000000
      //   )
      //   .signAndSend(aliceRelay); // NO SUDO FOR RELAY
      // console.log("res3", res3);
      // console.log("after", (await parachainOne.query.system.account(ALITH)).data.free.toHuman());
      // expect((await parachainOne.query.system.account(ALITH)).data.free.toHuman()).to.eq(
      //   "1.2078 MMOVR"
      // );
    });
  }
);

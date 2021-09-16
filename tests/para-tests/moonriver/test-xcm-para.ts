import Keyring from "@polkadot/keyring";
import { expect } from "chai";
import { BN } from "@polkadot/util";

import { ALITH, ALITH_PRIV_KEY, GENESIS_ACCOUNT_PRIVATE_KEY } from "../../util/constants";
import { describeParachain } from "../../util/setup-para-tests";
import { createBlockWithExtrinsicParachain, waitOneBlock } from "../../util/substrate-rpc";
import { ApiPromise } from "@polkadot/api";

const palletId = "0x6D6f646c617373746d6E67720000000000000000";

const assetMetadata = {
  name: "DOT",
  symbol: "DOT",
  decimals: new BN(12),
  isFrozen: false,
};
const sourceLocation = { XCM: { X1: "Parent" } };

// export async function waitOneBlock(api:ApiPromise){
//     return new Promise((res)=>{
//         api.derive.chain.subscribeNewHeads((header) => {
//             console.log(`One block elapsed:#${header.number}: ${header.author}`);
//             res()
//           });
//     })
// }

describeParachain(
  "XCM - receive_relay_asset_from_relay",
  { chain: "moonbase-local" },
  (context) => {
    it("should be accessible through web3", async function () {
      const keyring = new Keyring({ type: "sr25519" });
      const aliceRelay = keyring.addFromUri("//Alice");

      const alith = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");

      const parachainOne = context.polkadotApiParaone;
      const relayOne = context._polkadotApiRelaychains[0];

      // subscribe to all new headers (with extended info)
      context.polkadotApiParaone.derive.chain.subscribeNewHeads(async (header) => {
        console.log(`#${header.number}: ${header.author}`);
        (await context.polkadotApiParaone.query.system.events.at(header.hash)).forEach((e, i) => {
          console.log(
            "event",
            // header.number,
            header.hash.toHex(),
            i,
            (e.toHuman() as any).event.method
          );
        });
      });
      // TODO: monitor relay events
      await new Promise((res) => setTimeout(res, 10000));

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
      console.log("DONE");
      //   await parachainOne.tx.sudo
      //     .sudo(parachainOne.tx.assetManager.registerAsset(sourceLocation, assetMetadata, new BN(1)))
      //     .signAndSend(alith);
      //   await waitOneBlock(parachainOne);
      // Look for assetId in events
      let assetId: string;
      eventsRegister.forEach((e) => {
        // console.log(e.toHuman());
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
        context,
        alith,
        parachainOne.tx.sudo.sudo(parachainOne.tx.assetManager.setAssetUnitsPerSecond(assetId, 0))
      );
      console.log("setAssetUnitsPerSecond DONE");
      // events.forEach((e) => console.log(e.toHuman()));
      expect(events[0].toHuman().method).to.eq("UnitsPerSecondChanged");
      expect(events[2].toHuman().method).to.eq("ExtrinsicSuccess");
      console.log("PARACHAIN SUCCESS");

      // check asset in storage
      const registeredAsset = await parachainOne.query.assets.asset(assetId);
      expect((registeredAsset.toHuman() as { owner: string }).owner).to.eq(palletId);

      // RELAYCHAIN
      //   const res3 = await relayOne.tx.xcmPallet
      //     .reserveTransferAssets(
      //       { X1: { Parachain: new BN(1000) } },
      //       { X1: { AccountKey20: { network: "Any", key: ALITH } } },
      //       [{ ConcreteFungible: { id: "Here", amount: new BN(1000000000000000) } }],
      //       new BN(4000000000)
      //     )
      //     .signAndSend(aliceRelay); // NO SUDO FOR RELAY
      const { events: eventsRelay } = await createBlockWithExtrinsicParachain(
        context,
        aliceRelay,
        relayOne.tx.xcmPallet.reserveTransferAssets(
          { X1: { Parachain: new BN(1000) } },
          { X1: { AccountKey20: { network: "Any", key: ALITH } } },
          [{ ConcreteFungible: { id: "Here", amount: new BN(1000000000000000) } }],
          new BN(4000000000)
        )
      );
      console.log("last call");
      eventsRelay.forEach((e) => {
        console.log(e.toHuman());
      });
      console.log("eventsRelay", eventsRelay.length);
      console.log("after", (await parachainOne.query.system.account(ALITH)).data.free.toHuman());
      //   expect((await parachainOne.query.system.account(ALITH)).data.free.toHuman()).to.eq(
      //     "1.2078 MUNIT"
      //   );
      expect(((await relayOne.query.system.account(aliceRelay)) as any).data.free.toHuman()).to.eq(
        "1.2078 MUNIT"
      );
    });
  }
);

import Keyring from "@polkadot/keyring";
import { KeyringPair } from "@polkadot/keyring/types";
import { expect } from "chai";
import { BN } from "@polkadot/util";

import { ALITH_PRIV_KEY, GLMR } from "../util/constants";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createBlockWithExtrinsic } from "../util/substrate-rpc";
import { customWeb3Request } from "../util/providers";

const palletId = "0x6D6f646c617373746d6E67720000000000000000";

const assetMetadata = {
  name: "DOT",
  symbol: "DOT",
  decimals: new BN(12),
  isFrozen: false,
};

//TODO Gorka showed me to think of this as { parents: 1, intherior: here }. Are these the same?
// const sourceLocation = { XCM: { X1: "Parent" } };
const sourceLocation = { XCM: { parents: 1, interior: "Here" } };

describeDevMoonbeam("Mock XCM - receive downward transfer", (context) => {
  let assetId: string;
  let alith: KeyringPair;

  //TODO this entire block copied from another test, make sure it is the right params for DOTs
  before("Should Register an asset and set unit per sec", async function () {
    const keyringEth = new Keyring({ type: "ethereum" });
    alith = keyringEth.addFromUri(ALITH_PRIV_KEY, null, "ethereum");

    // registerAsset
    const { events: eventsRegister } = await createBlockWithExtrinsic(
      context,
      alith,
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.assetManager.registerAsset(sourceLocation, assetMetadata, new BN(1))
      )
    );
    // Look for assetId in events
    // let assetId: string;
    eventsRegister.forEach((e) => {
      if (e.section.toString() === "assetManager") {
        assetId = e.data[0].toHex();
      }
    });
    assetId = assetId.replace(/,/g, "");

    // setAssetUnitsPerSecond
    const { events } = await createBlockWithExtrinsic(
      context,
      alith,
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.assetManager.setAssetUnitsPerSecond(assetId, 0)
      )
    );
    expect(events[1].method.toString()).to.eq("UnitsPerSecondChanged");
    expect(events[4].method.toString()).to.eq("ExtrinsicSuccess");

    // check asset in storage
    // console.log("assetid", assetId);
    const registeredAsset = (
      (await context.polkadotApi.query.assets.asset(assetId)) as any
    ).unwrap();
    // console.log(1, registeredAsset);
    expect(registeredAsset.owner.toHex()).to.eq(palletId.toLowerCase());
  });

  it.only("Should receive a downward transfer of 10 DOTs to Alith", async function () {
    // Send RPC call to inject XCM message
    // You can provide a message, but if you don't a downward transfer is the default
    await customWeb3Request(context.web3, "xcm_injectDownwardMessage", [[]]);

    // Create a block in which the XCM will be executed
    await context.createBlock();
    await context.createBlock();

    console.log("assetid", assetId);
    const registeredAsset = (
      (await context.polkadotApi.query.assets.asset(assetId)) as any
    ).unwrap();
    console.log(registeredAsset.toHuman());

    console.log("alith.address", alith.address);

    // Make sure the state (and events?) has ALITH's to DOT tokens
    expect(context.polkadotApi.query.assets.account(assetId, alith.address)).to.eq(10n * GLMR);
  });
});

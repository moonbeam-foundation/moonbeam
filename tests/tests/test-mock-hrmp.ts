import Keyring from "@polkadot/keyring";
import { KeyringPair } from "@polkadot/keyring/types";
import { expect } from "chai";
import { BN } from "@polkadot/util";

import { ALITH_PRIV_KEY } from "../util/constants";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createBlockWithExtrinsic } from "../util/substrate-rpc";
import { customWeb3Request } from "../util/providers";

const FOREIGN_TOKEN = 1_000_000_000_000n;

const palletId = "0x6D6f646c617373746d6E67720000000000000000";
const foreign_para_id = 2000;

const assetMetadata = {
  name: "FOREIGN",
  symbol: "FOREIGN",
  decimals: new BN(12),
  isFrozen: false,
};

const sourceLocation = { XCM: { parents: 1, interior: { X1: { Parachain: foreign_para_id } } } };

describeDevMoonbeam("Mock XCM - receive horiontal transfer", (context) => {
  let assetId: string;
  let alith: KeyringPair;

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
    const registeredAsset = (
      (await context.polkadotApi.query.assets.asset(assetId)) as any
    ).unwrap();
    expect(registeredAsset.owner.toHex()).to.eq(palletId.toLowerCase());
  });

  it("Should receive a horizontal transfer of 10 FOREIGNs to Alith", async function () {
    // Send RPC call to inject XCM message
    // You can provide a message, but if you don't a horizontal transfer is the default
    await customWeb3Request(context.web3, "xcm_injectHrmpMessage", [foreign_para_id, []]);

    // Create a block in which the XCM will be executed
    await context.createBlock();

    // Make sure the state has ALITH's foreign parachain tokens
    let alith_dot_balance = (
      await context.polkadotApi.query.assets.account(assetId, alith.address)
    ).balance.toBigInt();

    expect(alith_dot_balance).to.eq(10n * FOREIGN_TOKEN);
  });
});

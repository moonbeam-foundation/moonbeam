import Keyring from "@polkadot/keyring";
import { expect } from "chai";
import { ALITH, ALITH_PRIV_KEY } from "../util/constants";

import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createBlockWithExtrinsic } from "../util/substrate-rpc";

const TEST_ACCOUNT = "0x1111111111111111111111111111111111111111";

// A call from root (sudo) can make a transfer directly in pallet_evm
// A signed call cannot make a transfer directly in pallet_evm

describeDevMoonbeam("Pallet Maintenance Mode - no sudo", (context) => {
  let events;
  before("Try turning maintenance mode on", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    const alith = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
    ({ events } = await createBlockWithExtrinsic(
      context,
      alith,
      context.polkadotApi.tx.maintenanceMode.enterMaintenanceMode()
    ));
  });

  it("should fail without sudo", async function () {
    expect(events[5].toHuman().method).to.eq("ExtrinsicFailed");
    expect(await context.web3.eth.getBalance(TEST_ACCOUNT)).to.equal("0");
  });
});
describeDevMoonbeam("Pallet Maintenance Mode - with sudo", (context) => {
  let events;
  before("Try turning maintenance mode on", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    const alith = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");

    ({ events } = await createBlockWithExtrinsic(
      context,
      alith,
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.maintenanceMode.enterMaintenanceMode()
      )
    ));

    for (let i = 0; i < 2400; i++) {
      await context.createBlock();
    }
  });

  it.only("should succeed with sudo", async function () {
    expect(events[3].toHuman().method).to.eq("ExtrinsicSuccess");
    console.log(await context.polkadotApi.query.maintenanceMode.maintenanceMode());
    console.log((await context.polkadotApi.query.maintenanceMode.maintenanceMode()).toHuman());
    expect((await context.polkadotApi.query.maintenanceMode.maintenanceMode()).toHuman()).to.equal(
      true
    );
  });
});

describeDevMoonbeam("Pallet Maintenance Mode - remark without maintenance mode", (context) => {
  let events, extrinsic;
  before("Try turning maintenance mode on", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    const alith = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");

    ({ events, extrinsic } = await createBlockWithExtrinsic(
      context,
      alith,
      context.polkadotApi.tx.system.remark("0x07")
    ));
  });

  it.only("should succeed with sudo", async function () {
    events.forEach((e) => {
      console.log(e.toHuman());
    });
    console.log("extrinsic", JSON.stringify(extrinsic, null, 2));
    expect(events[5].method.toString()).to.eq("ExtrinsicSuccess");
  });
});

describeDevMoonbeam("Pallet Maintenance Mode - no remark with maintenance mode", (context) => {
  let events;
  before("Try turning maintenance mode on", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    const alith = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");

    await createBlockWithExtrinsic(
      context,
      alith,
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.maintenanceMode.enterMaintenanceMode()
      )
    );

    ({ events } = await createBlockWithExtrinsic(
      context,
      alith,
      context.polkadotApi.tx.system.remark("0x07")
    ));
  });

  it.only("shouldn't succeed with sudo", async function () {
    events.forEach((e) => {
      console.log(e.toHuman());
    });
    expect(events[5].method.toString()).to.eq("ExtrinsicFailed");
  });
});

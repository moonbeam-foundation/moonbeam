import { expect } from "chai";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import Keyring from "@polkadot/keyring";
import {
  ALITH_PRIVATE_KEY,
  BALTATHAR_PRIVATE_KEY,
  CHARLETH_PRIVATE_KEY,
  CHARLETH_ADDRESS,
} from "../../util/constants";
import { expectBalanceDifference } from "../../util/balances";
import { substrateTransaction } from "../../util/transactions";
const debug = require("debug")("test:proxy");

// In these tests Alith will allow Baltathar to perform calls on her behalf.
// Charleth is used as a target account when making transfers.
const keyring = new Keyring({ type: "ethereum" });
const alith = keyring.addFromUri(ALITH_PRIVATE_KEY, null, "ethereum");
const baltathar = keyring.addFromUri(BALTATHAR_PRIVATE_KEY, null, "ethereum");
const charleth = keyring.addFromUri(CHARLETH_PRIVATE_KEY, null, "ethereum");

describeDevMoonbeam("Pallet proxy - shouldn't accept unknown proxy", (context) => {
  it("shouldn't accept unknown proxy", async function () {
    await expectBalanceDifference(
      context,
      CHARLETH_ADDRESS,
      0,
      async () => {
        const events = await substrateTransaction(
          context,
          baltathar,
          context.polkadotApi.tx.proxy.proxy(
            alith.address,
            null,
            context.polkadotApi.tx.balances.transfer(charleth.address, 100)
          )
        );
        expect(events[5].method).to.be.eq("ExtrinsicFailed");
      },
      expect
    );
  });
});

describeDevMoonbeam("Pallet proxy - should accept known proxy", (context) => {
  it("should accept known proxy", async () => {
    await expectBalanceDifference(
      context,
      CHARLETH_ADDRESS,
      100,
      async () => {
        const events = await substrateTransaction(
          context,
          alith,
          context.polkadotApi.tx.proxy.addProxy(baltathar.address, "Any", 0)
        );
        expect(events[2].method).to.be.eq("ProxyAdded");
        expect(events[2].data[2].toString()).to.be.eq("Any"); //ProxyType
        expect(events[7].method).to.be.eq("ExtrinsicSuccess");

        const events2 = await substrateTransaction(
          context,
          baltathar,
          context.polkadotApi.tx.proxy.proxy(
            alith.address,
            null,
            context.polkadotApi.tx.balances.transfer(charleth.address, 100)
          )
        );
        expect(events2[2].method).to.be.eq("ProxyExecuted");
        expect(events2[2].data[0].toString()).to.be.eq("Ok");
        expect(events2[5].method).to.be.eq("ExtrinsicSuccess");
      },
      expect
    );
  });
});

describeDevMoonbeam("Pallet proxy - shouldn't accept removed proxy", (context) => {
  it("shouldn't accept removed proxy", async () => {
    await expectBalanceDifference(
      context,
      CHARLETH_ADDRESS,
      0,
      async () => {
        const events = await substrateTransaction(
          context,
          alith,
          context.polkadotApi.tx.proxy.addProxy(baltathar.address, "Any", 0)
        );
        expect(events[7].method).to.be.eq("ExtrinsicSuccess");

        const events2 = await substrateTransaction(
          context,
          alith,
          context.polkadotApi.tx.proxy.removeProxy(baltathar.address, "Any", 0)
        );
        expect(events2[4].method).to.be.eq("ExtrinsicSuccess");

        const events3 = await substrateTransaction(
          context,
          baltathar,
          context.polkadotApi.tx.proxy.proxy(
            alith.address,
            null,
            context.polkadotApi.tx.balances.transfer(charleth.address, 100)
          )
        );
        expect(events3[3].method).to.be.eq("ExtrinsicFailed");
      },
      expect
    );
  });
});

describeDevMoonbeam("Pallet proxy - shouldn't accept instant for delayed proxy", (context) => {
  it("shouldn't accept instant for delayed proxy", async () => {
    await expectBalanceDifference(
      context,
      CHARLETH_ADDRESS,
      0,
      async () => {
        const events = await substrateTransaction(
          context,
          alith,
          context.polkadotApi.tx.proxy.addProxy(baltathar.address, "Any", 2)
        );
        expect(events[7].method).to.be.eq("ExtrinsicSuccess");

        const events2 = await substrateTransaction(
          context,
          baltathar,
          context.polkadotApi.tx.proxy.proxy(
            alith.address,
            null,
            context.polkadotApi.tx.balances.transfer(charleth.address, 100)
          )
        );
        expect(events2[3].method).to.be.eq("ExtrinsicFailed");
      },
      expect
    );
  });
});

describeDevMoonbeam("Pallet proxy - shouldn't accept early delayed proxy", (context) => {
  it("shouldn't accept early delayed proxy", async () => {
    await expectBalanceDifference(
      context,
      CHARLETH_ADDRESS,
      0,
      async () => {
        const events = await substrateTransaction(
          context,
          alith,
          context.polkadotApi.tx.proxy.addProxy(baltathar.address, "Any", 6)
        );
        events.forEach((event) => debug(`1${event.method}(${event.data})`));
        expect(events[7].method).to.be.eq("ExtrinsicSuccess");

        const transfer = context.polkadotApi.tx.balances.transfer(charleth.address, 100);

        const events2 = await substrateTransaction(
          context,
          baltathar,
          context.polkadotApi.tx.proxy.announce(alith.address, transfer.hash)
        );
        events2.forEach((event) => debug(`2${event.method}(${event.data})`));
        expect(events2[2].method).to.be.eq("Announced");
        expect(events2[5].method).to.be.eq("ExtrinsicSuccess");

        // Too early.
        const events3 = await substrateTransaction(
          context,
          baltathar,
          context.polkadotApi.tx.proxy.proxyAnnounced(
            baltathar.address,
            alith.address,
            null,
            transfer
          )
        );
        events3.forEach((event) => debug(`3${event.method}(${event.data})`));
        expect(events3[3].method).to.be.eq("ExtrinsicFailed");
      },
      expect
    );
  });
});

describeDevMoonbeam("Pallet proxy - should accept on-time delayed proxy", (context) => {
  it("should accept on-time delayed proxy ", async () => {
    await expectBalanceDifference(
      context,
      CHARLETH_ADDRESS,
      100,
      async () => {
        const events = await substrateTransaction(
          context,
          alith,
          context.polkadotApi.tx.proxy.addProxy(baltathar.address, "Any", 2)
        );
        events.forEach((e) => {
          debug(e.toHuman());
        });
        expect(events[7].method).to.be.eq("ExtrinsicSuccess");

        // Build transaction
        const transfer = context.polkadotApi.tx.balances.transfer(charleth.address, 100);
        const u8a = transfer.method.toU8a();
        const transfer_hash = transfer.registry.hash(u8a).toHex();

        const events2 = await substrateTransaction(
          context,
          baltathar,
          context.polkadotApi.tx.proxy.announce(alith.address, transfer_hash)
        );
        events2.forEach((event) => debug(`${event.method}(${event.data})`));
        expect(events2[2].method).to.be.eq("Announced");
        expect(events2[2].data[2].toHex()).to.eq(transfer_hash);
        expect(events2[5].method).to.be.eq("ExtrinsicSuccess");

        await context.createBlock();
        await context.createBlock();

        // On time.
        const events3 = await substrateTransaction(
          context,
          baltathar,
          context.polkadotApi.tx.proxy.proxyAnnounced(
            baltathar.address,
            alith.address,
            null,
            transfer
          )
        );
        debug("------");
        events3.forEach((event) => debug(`${event.method}(${event.data})`));
        expect(events3[1].method).not.to.be.eq("ExtrinsicFailed");
      },
      expect
    );
  });
});

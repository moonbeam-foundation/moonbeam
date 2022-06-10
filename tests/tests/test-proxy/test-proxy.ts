import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { expectBalanceDifference } from "../../util/balances";
import { alith, baltathar, charleth } from "../../util/accounts";
const debug = require("debug")("test:proxy");

// In these tests Alith will allow Baltathar to perform calls on her behalf.
// Charleth is used as a target account when making transfers.

describeDevMoonbeam("Pallet proxy - shouldn't accept unknown proxy", (context) => {
  it("shouldn't accept unknown proxy", async function () {
    await expectBalanceDifference(
      context,
      charleth.address,
      0,
      async () => {
        const {
          result: { events },
        } = await context.createBlock(
          context.polkadotApi.tx.proxy
            .proxy(
              alith.address,
              null,
              context.polkadotApi.tx.balances.transfer(charleth.address, 100)
            )
            .signAsync(baltathar)
        );
        expect(events[5].event.method).to.be.eq("ExtrinsicFailed");
      },
      expect
    );
  });
});

describeDevMoonbeam("Pallet proxy - should accept known proxy", (context) => {
  it("should accept known proxy", async () => {
    await expectBalanceDifference(
      context,
      charleth.address,
      100,
      async () => {
        const {
          result: { events },
        } = await context.createBlock(
          context.polkadotApi.tx.proxy.addProxy(baltathar.address, "Any", 0)
        );
        expect(events[2].event.method).to.be.eq("ProxyAdded");
        expect(events[2].event.data[2].toString()).to.be.eq("Any"); //ProxyType
        expect(events[7].event.method).to.be.eq("ExtrinsicSuccess");

        const {
          result: { events: events2 },
        } = await context.createBlock(
          context.polkadotApi.tx.proxy
            .proxy(
              alith.address,
              null,
              context.polkadotApi.tx.balances.transfer(charleth.address, 100)
            )
            .signAsync(baltathar)
        );
        expect(events2[2].event.method).to.be.eq("ProxyExecuted");
        expect(events2[2].event.data[0].toString()).to.be.eq("Ok");
        expect(events2[5].event.method).to.be.eq("ExtrinsicSuccess");
      },
      expect
    );
  });
});

describeDevMoonbeam("Pallet proxy - shouldn't accept removed proxy", (context) => {
  it("shouldn't accept removed proxy", async () => {
    await expectBalanceDifference(
      context,
      charleth.address,
      0,
      async () => {
        const {
          result: { events },
        } = await context.createBlock(
          context.polkadotApi.tx.proxy.addProxy(baltathar.address, "Any", 0)
        );
        expect(events[7].event.method).to.be.eq("ExtrinsicSuccess");

        const {
          result: { events: events2 },
        } = await context.createBlock(
          context.polkadotApi.tx.proxy.removeProxy(baltathar.address, "Any", 0)
        );
        expect(events2[5].event.method).to.be.eq("ExtrinsicSuccess");

        const {
          result: { events: events3 },
        } = await context.createBlock(
          context.polkadotApi.tx.proxy.proxy(
            alith.address,
            null,
            context.polkadotApi.tx.balances.transfer(charleth.address, 100)
          )
        );
        expect(events3[3].event.method).to.be.eq("ExtrinsicFailed");
      },
      expect
    );
  });
});

describeDevMoonbeam("Pallet proxy - shouldn't accept instant for delayed proxy", (context) => {
  it("shouldn't accept instant for delayed proxy", async () => {
    await expectBalanceDifference(
      context,
      charleth.address,
      0,
      async () => {
        const {
          result: { events },
        } = await context.createBlock(
          context.polkadotApi.tx.proxy.addProxy(baltathar.address, "Any", 2)
        );
        expect(events[7].event.method).to.be.eq("ExtrinsicSuccess");

        const {
          result: { events: events2 },
        } = await context.createBlock(
          context.polkadotApi.tx.proxy
            .proxy(
              alith.address,
              null,
              context.polkadotApi.tx.balances.transfer(charleth.address, 100)
            )
            .signAsync(baltathar)
        );
        expect(events2[3].event.method).to.be.eq("ExtrinsicFailed");
      },
      expect
    );
  });
});

describeDevMoonbeam("Pallet proxy - shouldn't accept early delayed proxy", (context) => {
  it("shouldn't accept early delayed proxy", async () => {
    await expectBalanceDifference(
      context,
      charleth.address,
      0,
      async () => {
        const {
          result: { events },
        } = await context.createBlock(
          context.polkadotApi.tx.proxy.addProxy(baltathar.address, "Any", 6)
        );
        events.forEach(({ event }) => debug(`1${event.method}(${event.data})`));
        expect(events[7].event.method).to.be.eq("ExtrinsicSuccess");

        const transfer = context.polkadotApi.tx.balances.transfer(charleth.address, 100);

        const {
          result: { events: events2 },
        } = await context.createBlock(
          context.polkadotApi.tx.proxy.announce(alith.address, transfer.hash).signAsync(baltathar)
        );
        events2.forEach(({ event }) => debug(`2${event.method}(${event.data})`));
        expect(events2[2].event.method).to.be.eq("Announced");
        expect(events2[5].event.method).to.be.eq("ExtrinsicSuccess");

        // Too early.
        const {
          result: { events: events3 },
        } = await context.createBlock(
          context.polkadotApi.tx.proxy
            .proxyAnnounced(baltathar.address, alith.address, null, transfer)
            .signAsync(baltathar)
        );
        events3.forEach(({ event }) => debug(`3${event.method}(${event.data})`));
        expect(events3[3].event.method).to.be.eq("ExtrinsicFailed");
      },
      expect
    );
  });
});

describeDevMoonbeam("Pallet proxy - should accept on-time delayed proxy", (context) => {
  it("should accept on-time delayed proxy ", async () => {
    await expectBalanceDifference(
      context,
      charleth.address,
      100,
      async () => {
        const {
          result: { events },
        } = await context.createBlock(
          context.polkadotApi.tx.proxy.addProxy(baltathar.address, "Any", 2)
        );
        expect(events[7].event.method).to.be.eq("ExtrinsicSuccess");

        // Build transaction
        const transfer = context.polkadotApi.tx.balances.transfer(charleth.address, 100);
        const u8a = transfer.method.toU8a();
        const transfer_hash = transfer.registry.hash(u8a).toHex();

        const {
          result: { events: events2 },
        } = await context.createBlock(
          context.polkadotApi.tx.proxy.announce(alith.address, transfer_hash).signAsync(baltathar)
        );
        expect(events2[2].event.method).to.be.eq("Announced");
        expect(events2[2].event.data[2].toHex()).to.eq(transfer_hash);
        expect(events2[5].event.method).to.be.eq("ExtrinsicSuccess");

        await context.createBlock();
        await context.createBlock();

        // On time.
        const {
          result: { events: events3 },
        } = await context.createBlock(
          context.polkadotApi.tx.proxy
            .proxyAnnounced(baltathar.address, alith.address, null, transfer)
            .signAsync(baltathar)
        );
        expect(events3[1].event.method).not.to.be.eq("ExtrinsicFailed");
      },
      expect
    );
  });
});

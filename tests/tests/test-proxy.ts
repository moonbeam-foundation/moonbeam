import { expect } from "chai";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import Keyring from "@polkadot/keyring";
import { Event } from "@polkadot/types/interfaces";
import {
  ALITH_PRIVATE_KEY,
  BALTATHAR_PRIVATE_KEY,
  CHARLETH_PRIVATE_KEY,
  CHARLETH_ADDRESS,
} from "../util/constants";
import { createBlockWithExtrinsic, logEvents } from "../util/substrate-rpc";
const debug = require("debug")("test:proxy");

// In these tests Alith will allow Baltathar to perform calls on her behalf.
// Charleth is used as a target account when making transfers.
const keyring = new Keyring({ type: "ethereum" });
const alith = keyring.addFromUri(ALITH_PRIVATE_KEY, null, "ethereum");
const baltathar = keyring.addFromUri(BALTATHAR_PRIVATE_KEY, null, "ethereum");
const charleth = keyring.addFromUri(CHARLETH_PRIVATE_KEY, null, "ethereum");

/// Sign and sand Substrate transaction then create a block.
/// Will provide events emited by the transaction to check if they match what is expected.
async function substrateTransaction(
  context,
  sender,
  polkadotCall,
  inBlockCallback: (events: Event[]) => void
) {
  await new Promise(async (resolve) => {
    const { events } = await createBlockWithExtrinsic(context, sender, polkadotCall);
    await inBlockCallback(events);
    resolve(events);
  });
}

/// Fetch balance of provided account before and after the inner function is executed and
/// check it matches expected difference.
async function expectBalanceDifference(context, address, diff, inner) {
  const balance_before = await context.web3.eth.getBalance(address);

  await inner();

  const balance_after = await context.web3.eth.getBalance(CHARLETH_ADDRESS);
  expect(BigInt(balance_after)).to.be.eq(BigInt(balance_before) + BigInt(diff));
}

describeDevMoonbeam("Pallet proxy - shouldn't accept unknown proxy", (context) => {
  it("shouldn't accept unknown proxy", async () => {
    await expectBalanceDifference(context, CHARLETH_ADDRESS, 0, async () => {
      await substrateTransaction(
        context,
        baltathar,
        context.polkadotApi.tx.proxy.proxy(
          alith.address,
          null,
          context.polkadotApi.tx.balances.transfer(charleth.address, 100)
        ),
        (events) => {
          expect(events[3].method).to.be.eq("ExtrinsicFailed");
        }
      );
    });
  });
});

describeDevMoonbeam("Pallet proxy - should accept known proxy", (context) => {
  it("should accept known proxy", async () => {
    await expectBalanceDifference(context, CHARLETH_ADDRESS, 100, async () => {
      await substrateTransaction(
        context,
        alith,
        context.polkadotApi.tx.proxy.addProxy(baltathar.address, "Any", 0),
        (events) => {
          expect(events[4].method).to.be.eq("ExtrinsicSuccess");
        }
      );

      await substrateTransaction(
        context,
        baltathar,
        context.polkadotApi.tx.proxy.proxy(
          alith.address,
          null,
          context.polkadotApi.tx.balances.transfer(charleth.address, 100)
        ),
        (events) => {
          expect(events[3].method).to.be.eq("ExtrinsicSuccess");
        }
      );
    });
  });
});

describeDevMoonbeam("Pallet proxy - should accept removed proxy", (context) => {
  it("should accept removed proxy", async () => {
    await expectBalanceDifference(context, CHARLETH_ADDRESS, 0, async () => {
      await substrateTransaction(
        context,
        alith,
        context.polkadotApi.tx.proxy.addProxy(baltathar.address, "Any", 0),
        (events) => {
          expect(events[4].method).to.be.eq("ExtrinsicSuccess");
        }
      );

      await substrateTransaction(
        context,
        alith,
        context.polkadotApi.tx.proxy.removeProxy(baltathar.address, "Any", 0),
        (events) => {
          expect(events[2].method).to.be.eq("ExtrinsicSuccess");
        }
      );

      await substrateTransaction(
        context,
        baltathar,
        context.polkadotApi.tx.proxy.proxy(
          alith.address,
          null,
          context.polkadotApi.tx.balances.transfer(charleth.address, 100)
        ),
        (events) => {
          expect(events[1].method).to.be.eq("ExtrinsicFailed");
        }
      );
    });
  });
});

describeDevMoonbeam("Pallet proxy - shouldn't accept instant for delayed proxy", (context) => {
  it("shouldn't accept instant for delayed proxy", async () => {
    await expectBalanceDifference(context, CHARLETH_ADDRESS, 0, async () => {
      await substrateTransaction(
        context,
        alith,
        context.polkadotApi.tx.proxy.addProxy(baltathar.address, "Any", 2),
        (events) => {
          expect(events[4].method).to.be.eq("ExtrinsicSuccess");
        }
      );

      await substrateTransaction(
        context,
        baltathar,
        context.polkadotApi.tx.proxy.proxy(
          alith.address,
          null,
          context.polkadotApi.tx.balances.transfer(charleth.address, 100)
        ),
        (events) => {
          expect(events[1].method).to.be.eq("ExtrinsicFailed");
        }
      );
    });
  });
});

describeDevMoonbeam("Pallet proxy - shouldn't accept early delayed proxy", (context) => {
  it("shouldn't accept early delayed proxy", async () => {
    await expectBalanceDifference(context, CHARLETH_ADDRESS, 0, async () => {
      await substrateTransaction(
        context,
        alith,
        context.polkadotApi.tx.proxy.addProxy(baltathar.address, "Any", 6),
        (events) => {
          events.forEach((event) => debug(`1${event.method}(${event.data})`));
          expect(events[4].method).to.be.eq("ExtrinsicSuccess");
        }
      );

      const transfer = context.polkadotApi.tx.balances.transfer(charleth.address, 100);

      await substrateTransaction(
        context,
        baltathar,
        context.polkadotApi.tx.proxy.announce(alith.address, transfer.hash),
        (events) => {
          events.forEach((event) => debug(`2${event.method}(${event.data})`));
          expect(events[1].method).to.be.eq("Announced");
          expect(events[3].method).to.be.eq("ExtrinsicSuccess");
        }
      );

      // Too early.
      await substrateTransaction(
        context,
        baltathar,
        context.polkadotApi.tx.proxy.proxyAnnounced(
          baltathar.address,
          alith.address,
          null,
          transfer
        ),
        (events) => {
          events.forEach((event) => debug(`3${event.method}(${event.data})`));
          expect(events[1].method).to.be.eq("ExtrinsicFailed");
        }
      );
    });
  });
});

describeDevMoonbeam("Pallet proxy - should accept on-time delayed proxy", (context) => {
  it("should accept on-time delayed proxy ", async () => {
    await expectBalanceDifference(context, CHARLETH_ADDRESS, 100, async () => {
      await substrateTransaction(
        context,
        alith,
        context.polkadotApi.tx.proxy.addProxy(baltathar.address, "Any", 2),
        (events) => {
          events.forEach((e) => {
            debug(e.toHuman());
          });
          expect(events[4].method).to.be.eq("ExtrinsicSuccess");
        }
      );

      // Build transaction
      const transfer = context.polkadotApi.tx.balances.transfer(charleth.address, 100);
      const u8a = transfer.method.toU8a();
      const transfer_hash = transfer.registry.hash(u8a).toHex();

      await substrateTransaction(
        context,
        baltathar,
        context.polkadotApi.tx.proxy.announce(alith.address, transfer_hash),
        (events) => {
          events.forEach((event) => debug(`${event.method}(${event.data})`));
          expect(events[1].method).to.be.eq("Announced");
          expect(events[1].data[2].toHex()).to.eq(transfer_hash);
          expect(events[3].method).to.be.eq("ExtrinsicSuccess");
        }
      );

      await context.createBlock();
      await context.createBlock();

      // On time.
      await substrateTransaction(
        context,
        baltathar,
        context.polkadotApi.tx.proxy.proxyAnnounced(
          baltathar.address,
          alith.address,
          null,
          transfer
        ),
        (events) => {
          debug("------");
          events.forEach((event) => debug(`${event.method}(${event.data})`));
          expect(events[1].method).not.to.be.eq("ExtrinsicFailed");
        }
      );
    });
  });
});

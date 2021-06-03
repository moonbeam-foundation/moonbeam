import { expect } from "chai";
import { customWeb3Request } from "../util/providers";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import Keyring from "@polkadot/keyring";
import {
  ALITH_PRIVATE_KEY,
  BALTATHAR_PRIVATE_KEY,
  CHARLETH_PRIVATE_KEY,
  CHARLETH_ADDRESS,
} from "../util/constants";
import { blake2AsU8a } from "@polkadot/util-crypto";

// In these tests Alith will allow Baltathar to perform calls on her behalf.
// Charleth is used as a target account when making transfers.
const keyring = new Keyring({ type: "ethereum" });
const alith = keyring.addFromUri(ALITH_PRIVATE_KEY, null, "ethereum");
const baltathar = keyring.addFromUri(BALTATHAR_PRIVATE_KEY, null, "ethereum");
const charleth = keyring.addFromUri(CHARLETH_PRIVATE_KEY, null, "ethereum");

/// Sign and sand Substrate transaction then create a block.
/// Will provide events emited by the transaction to check if they match what is expected.
async function substrateTransaction(context, sender, polkadotCall, inBlockCallback) {
  await new Promise(async (resolve) => {
    const unsub = await polkadotCall.signAndSend(sender, ({ events = [], status }) => {
      if (status.isInBlock) {
        inBlockCallback(events);
        unsub();
        resolve(null);
      }
    });

    await context.createBlock();
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
          expect(events[3].event.method).to.be.eq("ExtrinsicFailed");
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
          expect(events[4].event.method).to.be.eq("ExtrinsicSuccess");
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
          expect(events[3].event.method).to.be.eq("ExtrinsicSuccess");
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
          expect(events[4].event.method).to.be.eq("ExtrinsicSuccess");
        }
      );

      await substrateTransaction(
        context,
        alith,
        context.polkadotApi.tx.proxy.removeProxy(baltathar.address, "Any", 0),
        (events) => {
          expect(events[2].event.method).to.be.eq("ExtrinsicSuccess");
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
          expect(events[1].event.method).to.be.eq("ExtrinsicFailed");
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
          expect(events[4].event.method).to.be.eq("ExtrinsicSuccess");
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
          expect(events[1].event.method).to.be.eq("ExtrinsicFailed");
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
        context.polkadotApi.tx.proxy.addProxy(baltathar.address, "Any", 2),
        (events) => {
          expect(events[4].event.method).to.be.eq("ExtrinsicSuccess");
        }
      );

      const transfer = context.polkadotApi.tx.balances.transfer(charleth.address, 100);

      await substrateTransaction(
        context,
        baltathar,
        context.polkadotApi.tx.proxy.announce(alith.address, transfer.hash),
        (events) => {
          expect(events[1].event.method).to.be.eq("Announced");
          expect(events[3].event.method).to.be.eq("ExtrinsicSuccess");
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
          events.forEach((event) => console.log(`${event.event.method}(${event.event.data})`));
          expect(events[1].event.method).to.be.eq("ExtrinsicFailed");
        }
      );
    });
  });
});

describeDevMoonbeam("Pallet proxy - should accept on-time delayed proxy", (context) => {
  it("should accept on-time delayed proxy", async () => {
    await expectBalanceDifference(context, CHARLETH_ADDRESS, 0, async () => {
      await substrateTransaction(
        context,
        alith,
        context.polkadotApi.tx.proxy.addProxy(baltathar.address, "Any", 0),
        (events) => {
          expect(events[4].event.method).to.be.eq("ExtrinsicSuccess");
        }
      );

      const transfer = context.polkadotApi.tx.balances.transfer(charleth.address, 100);
      const transfer_hash = blake2AsU8a(transfer.data, 256);
      console.log("data : ");
      console.log(transfer.data);
      console.log(transfer.toU8a());

      console.log("hash : ");
      console.log(transfer_hash);
      console.log(transfer.hash);

      await substrateTransaction(
        context,
        baltathar,
        context.polkadotApi.tx.proxy.announce(alith.address, transfer_hash),
        (events) => {
          events.forEach((event) => console.log(`${event.event.method}(${event.event.data})`));

          expect(events[1].event.method).to.be.eq("Announced");
          expect(events[1].event.data[2]).to.be.deep.eq(transfer_hash);
          expect(events[3].event.method).to.be.eq("ExtrinsicSuccess");
        }
      );

      await context.createBlock();
      await context.createBlock();
      await context.createBlock();
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
          console.log("------");
          events.forEach((event) => console.log(`${event.event.method}(${event.event.data})`));
          expect(events[1].event.method).not.to.be.eq("ExtrinsicFailed");
        }
      );
    });
  });
});

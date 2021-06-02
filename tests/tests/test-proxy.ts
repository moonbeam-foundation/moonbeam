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
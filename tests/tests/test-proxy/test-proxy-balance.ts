import { expect } from "chai";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import Keyring from "@polkadot/keyring";
import { Event } from "@polkadot/types/interfaces";
import {
  ALITH_PRIVATE_KEY,
  BALTATHAR_PRIVATE_KEY,
  CHARLETH_PRIVATE_KEY,
  CHARLETH_ADDRESS,
  BOB_AUTHOR_ID,
} from "../../util/constants";
import { createBlockWithExtrinsic, logEvents } from "../../util/substrate-rpc";
const debug = require("debug")("test:proxy");

// In these tests Alith will allow Baltathar to perform calls on her behalf.
// Charleth is used as a target account when making transfers.
const keyring = new Keyring({ type: "ethereum" });
const alith = keyring.addFromUri(ALITH_PRIVATE_KEY, null, "ethereum");
const baltathar = keyring.addFromUri(BALTATHAR_PRIVATE_KEY, null, "ethereum");
const charleth = keyring.addFromUri(CHARLETH_PRIVATE_KEY, null, "ethereum");

/// Sign and sand Substrate transaction then create a block.
/// Will provide events emited by the transaction to check if they match what is expected.
async function substrateTransaction(context, sender, polkadotCall): Promise<Event[]> {
  const { events } = await createBlockWithExtrinsic(context, sender, polkadotCall);
  return events;
}

/// Fetch balance of provided account before and after the inner function is executed and
/// check it matches expected difference.
async function expectBalanceDifference(context, address, diff, inner) {
  const balance_before = await context.web3.eth.getBalance(address);

  await inner();

  const balance_after = await context.web3.eth.getBalance(CHARLETH_ADDRESS);
  expect(BigInt(balance_after)).to.be.eq(BigInt(balance_before) + BigInt(diff));
}

describeDevMoonbeam("Proxy: Balances - should accept known proxy", (context) => {
  it.only("should accept known proxy", async () => {
    await expectBalanceDifference(context, CHARLETH_ADDRESS, 100, async () => {
      const events = await substrateTransaction(
        context,
        alith,
        // @ts-ignore
        context.polkadotApi.tx.proxy.addProxy(baltathar.address, "Balances", 0)
      );
      expect(events[2].method).to.be.eq("ProxyAdded");
      expect(events[2].data[2].toString()).to.be.eq("Balances"); //ProxyType
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
      events2.forEach((e) => {
        console.log(2);
        console.log(e.toHuman());
      });
      expect(events2[2].method).to.be.eq("ProxyExecuted");
      expect(events2[2].data[0].toString()).to.be.eq("Ok");
      expect(events2[5].method).to.be.eq("ExtrinsicSuccess");
    });
  });
});

describeDevMoonbeam("Proxy: Balances - shouldn't accept other proxy types", (context) => {
  it.only("shouldn't accept other proxy types", async () => {
    await expectBalanceDifference(context, CHARLETH_ADDRESS, 100, async () => {
      const events = await substrateTransaction(
        context,
        alith,
        // @ts-ignore
        context.polkadotApi.tx.proxy.addProxy(baltathar.address, "Balances", 0)
      );
      events.forEach((e) => {
        console.log(1);
        console.log(e.toHuman());
      });

      const events2 = await substrateTransaction(
        context,
        baltathar,
        context.polkadotApi.tx.proxy.proxy(
          alith.address,
          null,
          context.polkadotApi.tx.authorMapping.addAssociation(BOB_AUTHOR_ID)
        )
      );
      events2.forEach((e) => {
        console.log(2);
        console.log(e.toHuman());
      });
      //   expect(events2[2].method).to.be.eq("ProxyExecuted");
      //   expect(events2[2].data[0].toString()).to.be.eq("Ok");
      expect(events2[5].method).to.be.eq("ExtrinsicFailed");
    });
  });
});

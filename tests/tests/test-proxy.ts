import { expect } from "chai";
import { customWeb3Request } from "../util/providers";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import Keyring from "@polkadot/keyring";
import { GENESIS_ACCOUNT_PRIVATE_KEY } from "../util/constants";

function delay(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

describeDevMoonbeam("Pallet proxy", (context) => {
  it("shouldn't accept unknown proxy", async function () {
    const keyring = new Keyring({ type: "ethereum" });
    const genesisAccount = await keyring.addFromUri(GENESIS_ACCOUNT_PRIVATE_KEY, null, "ethereum");
    const alice = keyring.addFromUri(
      "sample split bamboo west visual approve brain fox arch impact relief smile"
    );

    // Give alice some money.
    await context.polkadotApi.tx.balances
      .transfer(alice.address, 2_000_000_000)
      .signAndSend(genesisAccount);
    await context.createBlock();

    // Don't allow proxy.

    // Proxy call
    const unsub = await context.polkadotApi.tx.proxy
      .proxy(
        genesisAccount.address,
        null,
        context.polkadotApi.tx.balances.transfer(alice.address, 100)
      )
      .signAndSend(alice, ({ events = [], status }) => {
        if (status.isInBlock) {
          expect(events[1].event.method).to.be.eq("ExtrinsicFailed");
          unsub();
        }
      });

    await context.createBlock();
    await delay(500);
  });

  it("should accept known proxy", async function () {
    const keyring = new Keyring({ type: "ethereum" });
    const genesisAccount = await keyring.addFromUri(GENESIS_ACCOUNT_PRIVATE_KEY, null, "ethereum");
    const alice = keyring.addFromUri(
      "sample split bamboo west visual approve brain fox arch impact relief smile"
    );

    // Give alice some money.
    await context.polkadotApi.tx.balances
      .transfer(alice.address, 2_000_000_000)
      .signAndSend(genesisAccount);
    await context.createBlock();

    // Allow proxy
    await context.polkadotApi.tx.proxy
      .addProxy(alice.address, "Any", 0)
      .signAndSend(genesisAccount);

    // Proxy call
    const unsub = await context.polkadotApi.tx.proxy
      .proxy(
        genesisAccount.address,
        null,
        context.polkadotApi.tx.balances.transfer(alice.address, 100)
      )
      .signAndSend(alice, ({ events = [], status }) => {
        if (status.isInBlock) {
          expect(events[3].event.method).to.be.eq("ExtrinsicSuccess");
          unsub();
        }
      });

    await context.createBlock();
    await delay(500);
  });
});

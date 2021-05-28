import { expect } from "chai";
import { customWeb3Request } from "../util/providers";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import Keyring from "@polkadot/keyring";
import {
  GENESIS_ACCOUNT_PRIVATE_KEY,
  ALITH_PRIVATE_KEY,
  BALTATHAR_PRIVATE_KEY,
  BALTATHAR_ADDRESS,
} from "../util/constants";

function delay(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

describeDevMoonbeam("Pallet proxy", (context) => {
  it("shouldn't accept unknown proxy", async function () {
    const keyring = new Keyring({ type: "ethereum" });
    const genesisAccount = await keyring.addFromUri(GENESIS_ACCOUNT_PRIVATE_KEY, null, "ethereum");
    const alith = keyring.addFromUri(ALITH_PRIVATE_KEY, null, "ethereum");
    const baltathar = keyring.addFromUri(BALTATHAR_PRIVATE_KEY, null, "ethereum");

    const balance_before = await await context.web3.eth.getBalance(BALTATHAR_ADDRESS);

    // Proxy call
    const unsub = await context.polkadotApi.tx.proxy
      .proxy(
        genesisAccount.address,
        null,
        context.polkadotApi.tx.balances.transfer(baltathar.address, 100)
      )
      .signAndSend(alith, ({ events = [], status }) => {
        if (status.isInBlock) {
          // Check proxy call failed.
          expect(events[3].event.method).to.be.eq("ExtrinsicFailed");
          unsub();
        }
      });

    await context.createBlock();
    await delay(500);

    const balance_after = await await context.web3.eth.getBalance(BALTATHAR_ADDRESS);

    // Check target balance didn't change.
    expect(balance_after).to.be.eq(balance_before);
  });

  it("should accept known proxy", async function () {
    const keyring = new Keyring({ type: "ethereum" });
    const genesisAccount = await keyring.addFromUri(GENESIS_ACCOUNT_PRIVATE_KEY, null, "ethereum");
    const alith = keyring.addFromUri(ALITH_PRIVATE_KEY, null, "ethereum");
    const baltathar = keyring.addFromUri(BALTATHAR_PRIVATE_KEY, null, "ethereum");

    const balance_before = await await context.web3.eth.getBalance(BALTATHAR_ADDRESS);

    // Allow proxy
    await context.polkadotApi.tx.proxy
      .addProxy(alith.address, "Any", 0)
      .signAndSend(genesisAccount);

    // Proxy call
    const unsub = await context.polkadotApi.tx.proxy
      .proxy(
        genesisAccount.address,
        null,
        context.polkadotApi.tx.balances.transfer(baltathar.address, 100)
      )
      .signAndSend(alith, ({ events = [], status }) => {
        if (status.isInBlock) {
          // Check proxy call succeeded.
          expect(events[3].event.method).to.be.eq("ExtrinsicSuccess");
          unsub();
        }
      });

    await context.createBlock();
    await delay(500);

    const balance_after = await await context.web3.eth.getBalance(BALTATHAR_ADDRESS);

    // Check target balance changed with correct amount.
    expect(BigInt(balance_after)).to.be.eq(BigInt(balance_before) + 100n);
  });
});

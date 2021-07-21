import { expect } from "chai";

import { ALITH, ALITH_PRIV_KEY } from "../../util/constants";
import { describeParachain } from "../../util/setup-para-tests";
import Keyring from "@polkadot/keyring";

const CALL_ERROR = "1010: Invalid Transaction: Transaction call is not expected";
const keyring = new Keyring({ type: "ethereum" });

describeParachain("Runtime filters", { chain: "moonriver-local" }, (context) => {

  let account;
  before("Starting Moonbeam Test Node", async function () {
    account = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
  });

  it("should not be able to transfer", async function () {
    try {
        await context.polkadotApi.tx.balances.transfer(ALITH, 123).signAndSend(account);
    } catch (e) {
        expect(e.message).to.equal(CALL_ERROR);
        return;
    }
    throw new Error("should not be able to transfer");
  });

  it("should not be able to claim rewards", async function () {
    try {
        await context.polkadotApi.tx.crowdloanRewards.claim().signAndSend(account);
    } catch (e) {
        expect(e.message).to.equal(CALL_ERROR);
        return;
    }
    throw new Error("should not be able to claim rewards");
  });

  it("should not be able to ethereum transact", async function () {
    let tx = 
    "0x0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000" + 
    "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000" + 
    "000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000" + 
    "0000000000000000000000000000000000001a000000000000008c69faf613b9f72dbb029bb5d5acf42742d214" + 
    "c79743507e75fdc8adecdee92801be4f58ff278ac61125a81a582a717d9c5d6554326c01b878297c6522b12282";
    try {
        await context.polkadotApi.tx.ethereum.transact(tx).signAndSend(account);
    } catch (e) {
        expect(e.message).to.equal(CALL_ERROR);
        return;
    }
    throw new Error("should not be able to ethereum transact");
  });

  it("should not be able to evm call", async function () {
    try {
        await context.polkadotApi.tx.evm.call(
            "0x0000000000000000000000000000000000000001",
            "0x0000000000000000000000000000000000000002",
            "0x01", 0, 0, 0, null
        ).signAndSend(account);
    } catch (e) {
        expect(e.message).to.equal(CALL_ERROR);
        return;
    }
    throw new Error("should not be able to evm call");
  });

  // This shouldn't fail?
  it("should not be able to authorFilter.setEligible", async function () {
    try {
        await context.polkadotApi.tx.authorFilter.setEligible(0).signAndSend(account);
    } catch (e) {
        expect(e.message).to.equal(CALL_ERROR);
        return;
    }
    throw new Error("should not be able to authorFilter.setEligible");
  });
});

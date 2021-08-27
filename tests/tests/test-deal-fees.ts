import { expect } from "chai";
import Keyring from "@polkadot/keyring";
import { TREASURY_ACCOUNT } from "../util/constants";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createTransfer } from "../util/transactions";

describeDevMoonbeam("20% of the fees should go to treasury", (context) => {
  const TEST_ACCOUNT = "0x1111111111111111111111111111111111111111";
  const keyring = new Keyring({ type: "ethereum" });

  it("20% of the fees should go to treasury", async () => {

    // Treasury account should be initially empty
    expect(await context.web3.eth.getBalance(TREASURY_ACCOUNT, 0)).to.equal(0n.toString());
    
    // We make an ethereum transaction, 20% of the fees should go to treasury.
    await context.createBlock({
      transactions: [await createTransfer(context.web3, TEST_ACCOUNT, 128)],
    }); 
    expect(await context.web3.eth.getBalance(TREASURY_ACCOUNT, 1)).to.equal("4200000000000");
  });
});

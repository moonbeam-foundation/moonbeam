import { expect } from "chai";

import { customRequest } from "../tests/util";
import { GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY, TEST_ACCOUNT } from "../tests/constants";
import {
  createAndFinalizeBlockWithFrontier,
  describeWithFrontier,
} from "../tests/util/testWithFrontier";

const FRONTIER_GENESIS_ACCOUNT_BALANCE = "340282366920938463463374607431768211455";

// This is an example of a Frontier test. It requires to have a clone of frontier in the same repo
// The binary needs to be built with `cargo build --no-default-features --features=manual-seal`
describeWithFrontier("Frontier RPC (Balance)", `frontier-specs.json`, (context) => {
  it("genesis balance is setup correctly (web3)", async function () {
    expect(await context.web3.eth.getBalance(GENESIS_ACCOUNT)).to.equal(
      FRONTIER_GENESIS_ACCOUNT_BALANCE
    );
  });
  it("balance to be updated after transfer", async function () {
    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        to: TEST_ACCOUNT,
        value: "0x200", // Must be higher than ExistentialDeposit (0)
        gasPrice: "0x01",
        gas: "0x100000",
      },
      GENESIS_ACCOUNT_PRIVATE_KEY
    );
    await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);
    await createAndFinalizeBlockWithFrontier(context.web3);
    expect(await context.web3.eth.getBalance(GENESIS_ACCOUNT)).to.equal(
      "340282366920938463463374607431768189943"
    );
    expect(await context.web3.eth.getBalance(TEST_ACCOUNT)).to.equal("512");
  });
});

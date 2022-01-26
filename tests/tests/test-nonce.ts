import { expect } from "chai";

import { GENESIS_ACCOUNT, TEST_ACCOUNT } from "../util/constants";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createTransfer } from "../util/transactions";
import { customWeb3Request } from "../util/providers";

describeDevMoonbeam("Nonce - Initial", (context) => {
  it("should be at 0 before using it", async function () {
    expect(await context.web3.eth.getTransactionCount(TEST_ACCOUNT)).to.eq(0);
  });

  it("should be at 0 for genesis account", async function () {
    expect(await context.web3.eth.getTransactionCount(GENESIS_ACCOUNT)).to.eq(0);
  });

  it("should stay at 0 before block is created", async function () {
    await customWeb3Request(context.web3, "eth_sendRawTransaction", [
      await createTransfer(context, GENESIS_ACCOUNT, 512),
    ]);
    expect(await context.web3.eth.getTransactionCount(GENESIS_ACCOUNT)).to.eq(0);
  });
});

describeDevMoonbeam("Nonce - Previous block", (context) => {
  before("Setup: Create block with transfer", async () => {
    await context.createBlock({
      transactions: [await createTransfer(context, TEST_ACCOUNT, 512)],
    });
  });
  it("should be at 0 after transferring", async function () {
    expect(await context.web3.eth.getTransactionCount(GENESIS_ACCOUNT, 0)).to.eq(0);
  });
});

describeDevMoonbeam("Nonce - Pending transaction", (context) => {
  before("Setup: Create block with transfer", async () => {
    await customWeb3Request(context.web3, "eth_sendRawTransaction", [
      await createTransfer(context, TEST_ACCOUNT, 512),
    ]);
  });
  it("should not increase transaction count", async function () {
    expect(await context.web3.eth.getTransactionCount(GENESIS_ACCOUNT)).to.eq(0);
  });
  it("should not increase transaction count in latest block", async function () {
    expect(await context.web3.eth.getTransactionCount(GENESIS_ACCOUNT, "latest")).to.eq(0);
  });
  it("should increase transaction count in pending block", async function () {
    expect(await context.web3.eth.getTransactionCount(GENESIS_ACCOUNT, "pending")).to.eq(1);
  });
});

describeDevMoonbeam("Nonce - Transferring", (context) => {
  before("Setup: Sending token", async function () {
    await context.createBlock({
      transactions: [await createTransfer(context, TEST_ACCOUNT, 512)],
    });
  });

  it("should increase the sender nonce", async function () {
    expect(await context.web3.eth.getTransactionCount(GENESIS_ACCOUNT)).to.eq(1);
  });

  it("should not increase the receiver nonce", async function () {
    expect(await context.web3.eth.getTransactionCount(TEST_ACCOUNT)).to.eq(0);
  });
});

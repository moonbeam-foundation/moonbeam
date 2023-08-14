import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { alith, baltathar } from "../../../util/accounts";
import { customWeb3Request } from "../../../util/providers";
import { describeDevMoonbeam } from "../../../util/setup-dev-tests";
import { createTransfer } from "../../../util/transactions";

describeDevMoonbeam("Ethereum Transaction - Initial Nonce", (context) => {
  it("should be at 0 before using it", async function () {
    expect(await context.web3.eth.getTransactionCount(baltathar.address)).to.eq(0);
  });

  it("should be at 0 for genesis account", async function () {
    expect(await context.web3.eth.getTransactionCount(alith.address)).to.eq(0);
  });

  it("should stay at 0 before block is created", async function () {
    await customWeb3Request(context.web3, "eth_sendRawTransaction", [
      await createTransfer(context, alith.address, 512),
    ]);
    expect(await context.web3.eth.getTransactionCount(alith.address)).to.eq(0);
  });
});

describeDevMoonbeam("Ethereum Transaction - Previous block nonce", (context) => {
  before("Setup: Create block with transfer", async () => {
    await context.createBlock(createTransfer(context, baltathar.address, 512));
  });
  it("should be at 0 after transferring", async function () {
    expect(await context.web3.eth.getTransactionCount(alith.address, 0)).to.eq(0);
  });
});

describeDevMoonbeam("Ethereum Transaction - Pending transaction nonce", (context) => {
  before("Setup: Create block with transfer", async () => {
    await customWeb3Request(context.web3, "eth_sendRawTransaction", [
      await createTransfer(context, baltathar.address, 512),
    ]);
  });
  it("should not increase transaction count", async function () {
    expect(await context.web3.eth.getTransactionCount(alith.address)).to.eq(0);
  });
  it("should not increase transaction count in latest block", async function () {
    expect(await context.web3.eth.getTransactionCount(alith.address, "latest")).to.eq(0);
  });
  it("should increase transaction count in pending block", async function () {
    expect(await context.web3.eth.getTransactionCount(alith.address, "pending")).to.eq(1);
  });
});

describeDevMoonbeam("Ethereum Transaction - Transferring Nonce", (context) => {
  before("Setup: Sending token", async function () {
    await context.createBlock(createTransfer(context, baltathar.address, 512));
  });

  it("should increase the sender nonce", async function () {
    expect(await context.web3.eth.getTransactionCount(alith.address)).to.eq(1);
  });

  it("should not increase the receiver nonce", async function () {
    expect(await context.web3.eth.getTransactionCount(baltathar.address)).to.eq(0);
  });
});

import { expect } from "chai";

import { baltathar, ALITH_GENESIS_TRANSFERABLE_BALANCE } from "../../util/accounts";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createTransfer } from "../../util/transactions";
import { customWeb3Request } from "../../util/providers";

describeDevMoonbeam("Ethereum Rpc pool errors - already known #1", (context) => {
  it("already known #1", async function () {
    const tx = await createTransfer(context, baltathar.address, 1, { nonce: 0 });
    await customWeb3Request(context.web3, "eth_sendRawTransaction", [tx]);
    const res_a2 = await customWeb3Request(context.web3, "eth_sendRawTransaction", [tx]);
    expect(res_a2.error).to.include({
      message: "already known",
    });
    await context.createBlock();
  });
});

describeDevMoonbeam("Ethereum Rpc pool errors - replacement transaction underpriced", (context) => {
  it("replacement transaction underpriced", async function () {
    const tx_1 = await createTransfer(context, baltathar.address, 1, {
      nonce: 0,
      gasPrice: 2_000_000_000,
    });
    const tx_2 = await createTransfer(context, baltathar.address, 1, {
      nonce: 0,
      gasPrice: 1_000_000_000,
    });
    await customWeb3Request(context.web3, "eth_sendRawTransaction", [tx_1]);
    const res_a2 = await customWeb3Request(context.web3, "eth_sendRawTransaction", [tx_2]);
    expect(res_a2.error).to.include({
      message: "replacement transaction underpriced",
    });
    await context.createBlock();
  });
});

describeDevMoonbeam("Ethereum Rpc pool errors - nonce too low", (context) => {
  it("nonce too low", async function () {
    const tx_1 = await createTransfer(context, baltathar.address, 1, { nonce: 0 });
    await context.createBlock(tx_1);
    const tx_2 = await createTransfer(context, baltathar.address, 2, { nonce: 0 });
    const res_a2 = await customWeb3Request(context.web3, "eth_sendRawTransaction", [tx_2]);
    expect(res_a2.error).to.include({
      message: "nonce too low",
    });
    await context.createBlock();
  });
});

describeDevMoonbeam("Ethereum Rpc pool errors - already known #2", (context) => {
  it("already known #2", async function () {
    const tx_1 = await createTransfer(context, baltathar.address, 1, { nonce: 0 });
    await context.createBlock(tx_1);
    const tx_2 = await createTransfer(context, baltathar.address, 1, { nonce: 0 });
    const res_a2 = await customWeb3Request(context.web3, "eth_sendRawTransaction", [tx_2]);
    expect(res_a2.error).to.include({
      message: "already known",
    });
  });
});

describeDevMoonbeam(
  "Ethereum Rpc pool errors - insufficient funds for gas * price + value",
  (context) => {
    it("insufficient funds for gas * price + value", async function () {
      const ZEROED_ADDRESS = "0x740153b27427ecfc353b49ea6ab4c2e564d4ca15";
      const ZEROED_PKEY = "0xbf2a9f29a7631116a1128e34fcf8817581fb3ec159ef2be004b459bc33f2ed2d";
      const tx = await createTransfer(context, baltathar.address, 1, {
        from: ZEROED_ADDRESS,
        privateKey: ZEROED_PKEY,
      });
      const res = await customWeb3Request(context.web3, "eth_sendRawTransaction", [tx]);
      expect(res.error).to.include({
        message: "insufficient funds for gas * price + value",
      });
    });
  }
);

describeDevMoonbeam("Ethereum Rpc pool errors - exceeds block gas limit", (context) => {
  it("exceeds block gas limit", async function () {
    const tx = await createTransfer(context, baltathar.address, 1, {
      nonce: 0,
      gas: 1_000_000_0000,
    });
    const res = await customWeb3Request(context.web3, "eth_sendRawTransaction", [tx]);
    expect(res.error).to.include({
      message: "exceeds block gas limit",
    });
  });
});

describeDevMoonbeam(
  "Ethereum Rpc pool errors - insufficient funds for gas * price + value",
  (context) => {
    it("insufficient funds for gas * price + value", async function () {
      const amount = ALITH_GENESIS_TRANSFERABLE_BALANCE - 21000n * 1_000_000_000n + 1n;
      const tx = await createTransfer(context, baltathar.address, amount, { nonce: 0 });
      const res = await customWeb3Request(context.web3, "eth_sendRawTransaction", [tx]);
      expect(res.error).to.include({
        message: "insufficient funds for gas * price + value",
      });
    });
  }
);

describeDevMoonbeam(
  "Ethereum Rpc pool errors - max priority fee per gas higher than max fee per gas",
  (context) => {
    it("max priority fee per gas higher than max fee per gast", async function () {
      const tx = await createTransfer(context, baltathar.address, 1, {
        nonce: 0,
        maxPriorityFeePerGas: 2_000_000_000,
      });
      const res = await customWeb3Request(context.web3, "eth_sendRawTransaction", [tx]);
      expect(res.error).to.include({
        message: "max priority fee per gas higher than max fee per gas",
      });
    });
  },
  "EIP1559"
);

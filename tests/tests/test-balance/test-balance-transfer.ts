import { expect } from "chai";
import { BN } from "@polkadot/util";
import { verifyLatestBlockFees } from "../../util/block";
import {
  GENESIS_ACCOUNT,
  GENESIS_ACCOUNT_BALANCE,
  ALITH,
  ALITH_PRIV_KEY,
  TEST_ACCOUNT,
} from "../../util/constants";

import { describeDevMoonbeam, describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import { createTransfer, createTransaction } from "../../util/transactions";
import { customWeb3Request } from "../../util/providers";

describeDevMoonbeamAllEthTxTypes("Balance transfer cost", (context) => {
  it("should cost 21000 * 1_000_000_000", async function () {
    const testAccount = "0x1111111111111111111111111111111111111111";
    await context.createBlock({
      transactions: [await createTransfer(context, testAccount, 0)],
    });

    expect(await context.web3.eth.getBalance(GENESIS_ACCOUNT, 1)).to.equal(
      (GENESIS_ACCOUNT_BALANCE - 21000n * 1_000_000_000n).toString()
    );
  });
});

describeDevMoonbeamAllEthTxTypes("Balance transfer", (context) => {
  const TEST_ACCOUNT = "0x1111111111111111111111111111111111111111";
  before("Create block with transfer to test account of 512", async () => {
    await customWeb3Request(context.web3, "eth_sendRawTransaction", [
      await createTransfer(context, TEST_ACCOUNT, 512),
    ]);
    expect(await context.web3.eth.getBalance(GENESIS_ACCOUNT, "pending")).to.equal(
      (GENESIS_ACCOUNT_BALANCE - 512n - 21000n * 1_000_000_000n).toString()
    );
    expect(await context.web3.eth.getBalance(TEST_ACCOUNT, "pending")).to.equal("512");
    await context.createBlock();
  });

  it("should decrease from account", async function () {
    // 21000 covers the cost of the transaction
    expect(await context.web3.eth.getBalance(GENESIS_ACCOUNT, 1)).to.equal(
      (GENESIS_ACCOUNT_BALANCE - 512n - 21000n * 1_000_000_000n).toString()
    );
  });

  it("should increase to account", async function () {
    expect(await context.web3.eth.getBalance(TEST_ACCOUNT, 0)).to.equal("0");
    expect(await context.web3.eth.getBalance(TEST_ACCOUNT, 1)).to.equal("512");
  });

  it("should reflect balance identically on polkadot/web3", async function () {
    const block1Hash = await context.polkadotApi.rpc.chain.getBlockHash(1);
    expect(await context.web3.eth.getBalance(GENESIS_ACCOUNT, 1)).to.equal(
      (
        (await context.polkadotApi.query.system.account.at(block1Hash, GENESIS_ACCOUNT)) as any
      ).data.free.toString()
    );
  });
});

describeDevMoonbeamAllEthTxTypes("Balance transfer - fees", (context) => {
  const TEST_ACCOUNT = "0x1111111111111111111111111111111111111111";
  before("Create block with transfer to test account of 512", async () => {
    await context.createBlock({
      transactions: [await createTransfer(context, TEST_ACCOUNT, 512)],
    });
  });
  it("should check latest block fees", async function () {
    await verifyLatestBlockFees(context, expect, BigInt(512));
  });
});

describeDevMoonbeam("Balance transfer - EIP1559 fees", (context) => {
  it("should handle max_fee_per_gas", async function () {

    const preBalance = BigInt(await context.web3.eth.getBalance(ALITH));
    // With this configuration no priority fee will be used, as the max_fee_per_gas is exactly the
    // base fee. Expect the balances to reflect this case.
    const maxFeePerGas = 1_000_000_000;
    const tx = await createTransaction(context, {
      from: ALITH,
      privateKey: ALITH_PRIV_KEY,
      value: "0x0",
      gas: "0x5208",
      maxFeePerGas: maxFeePerGas,
      maxPriorityFeePerGas: "0xBEBC200", // 0.2GWEI
      to: TEST_ACCOUNT,
      data: "0x",
    });
  
    const block = await context.createBlock({
      transactions: [tx],
    });
    const postBalance = BigInt(await context.web3.eth.getBalance(ALITH));
    const fee = BigInt(21_000 * maxFeePerGas);
    const actualPostBalance = preBalance - fee;

    expect(postBalance).to.be.eq(actualPostBalance);
  });
}, "EIP1559");

describeDevMoonbeam("Balance transfer - EIP1559 fees", (context) => {
  it("should use partial max_priority_fee_per_gas", async function () {

    const preBalance = BigInt(await context.web3.eth.getBalance(ALITH));
    // With this configuration only half of the priority fee will be used, as the max_fee_per_gas is
    // 2GWEI and the base fee is 1GWEI.
    const maxFeePerGas = 1_000_000_000 * 2;
    const tx = await createTransaction(context, {
      from: ALITH,
      privateKey: ALITH_PRIV_KEY,
      value: "0x0",
      gas: "0x5208",
      maxFeePerGas: maxFeePerGas,
      maxPriorityFeePerGas: maxFeePerGas,
      to: TEST_ACCOUNT,
      data: "0x",
    });
  
    const block = await context.createBlock({
      transactions: [tx],
    });
    const postBalance = BigInt(await context.web3.eth.getBalance(ALITH));
    const fee = BigInt(21_000 * maxFeePerGas);
    const actualPostBalance = preBalance - fee;

    expect(postBalance).to.be.eq(actualPostBalance);
  });
}, "EIP1559");

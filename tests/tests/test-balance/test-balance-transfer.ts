import "@moonbeam-network/api-augment";

import { expect } from "chai";
import Web3 from "web3";

import {
  alith,
  ALITH_GENESIS_LOCK_BALANCE,
  ALITH_GENESIS_TRANSFERABLE_BALANCE,
  baltathar,
  generateKeyringPair,
} from "../../util/accounts";
import { verifyLatestBlockFees } from "../../util/block";
import { customWeb3Request } from "../../util/providers";
import { describeDevMoonbeam, describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import {
  ALITH_TRANSACTION_TEMPLATE,
  createTransaction,
  createTransfer,
} from "../../util/transactions";

describeDevMoonbeamAllEthTxTypes("Balance transfer cost", (context) => {
  const randomAccount = generateKeyringPair();
  it("should cost 21000 * 1_000_000_000", async function () {
    await context.createBlock(createTransfer(context, randomAccount.address, 0));

    expect(await context.web3.eth.getBalance(alith.address, 1)).to.equal(
      (ALITH_GENESIS_TRANSFERABLE_BALANCE - 21000n * 1_000_000_000n).toString()
    );
  });
});

describeDevMoonbeamAllEthTxTypes("Balance transfer", (context) => {
  const randomAccount = generateKeyringPair();
  before("Create block with transfer to test account of 512", async () => {
    await context.createBlock();
    await customWeb3Request(context.web3, "eth_sendRawTransaction", [
      await createTransfer(context, randomAccount.address, 512),
    ]);
    expect(await context.web3.eth.getBalance(alith.address, "pending")).to.equal(
      (ALITH_GENESIS_TRANSFERABLE_BALANCE - 512n - 21000n * 1_000_000_000n).toString()
    );
    expect(await context.web3.eth.getBalance(randomAccount.address, "pending")).to.equal("512");
    await context.createBlock();
  });

  it("should decrease from account", async function () {
    // 21000 covers the cost of the transaction
    expect(await context.web3.eth.getBalance(alith.address, 2)).to.equal(
      (ALITH_GENESIS_TRANSFERABLE_BALANCE - 512n - 21000n * 1_000_000_000n).toString()
    );
  });

  it("should increase to account", async function () {
    expect(await context.web3.eth.getBalance(randomAccount.address, 1)).to.equal("0");
    expect(await context.web3.eth.getBalance(randomAccount.address, 2)).to.equal("512");
  });

  it("should reflect balance identically on polkadot/web3", async function () {
    const block1Hash = await context.polkadotApi.rpc.chain.getBlockHash(1);
    expect(await context.web3.eth.getBalance(alith.address, 1)).to.equal(
      (
        (
          await (await context.polkadotApi.at(block1Hash)).query.system.account(alith.address)
        ).data.free.toBigInt() - ALITH_GENESIS_LOCK_BALANCE
      ).toString()
    );
  });
});

describeDevMoonbeamAllEthTxTypes("Balance transfer - fees", (context) => {
  const randomAccount = generateKeyringPair();
  before("Create block with transfer to test account of 512", async () => {
    await context.createBlock(createTransfer(context, randomAccount.address, 512));
  });
  it("should check latest block fees", async function () {
    await verifyLatestBlockFees(context, BigInt(512));
  });
});

describeDevMoonbeam("Balance transfer - Multiple transfers", (context) => {
  it("should be successful", async function () {
    const { result } = await context.createBlock([
      createTransfer(context, baltathar.address, 10n ** 18n, { nonce: 0 }),
      createTransfer(context, baltathar.address, 10n ** 18n, { nonce: 1 }),
      createTransfer(context, baltathar.address, 10n ** 18n, { nonce: 2 }),
      createTransfer(context, baltathar.address, 10n ** 18n, { nonce: 3 }),
    ]);
    expect(result.filter((r) => r.successful)).to.be.length(4);
  });
});

describeDevMoonbeam(
  "Balance transfer - EIP1559 fees",
  (context) => {
    it("should handle max_fee_per_gas", async function () {
      const randomAccount = generateKeyringPair();
      const preBalance = BigInt(await context.web3.eth.getBalance(alith.address));
      // With this configuration no priority fee will be used, as the max_fee_per_gas is exactly the
      // base fee. Expect the balances to reflect this case.
      const maxFeePerGas = 1_000_000_000;

      await context.createBlock(
        createTransaction(context, {
          ...ALITH_TRANSACTION_TEMPLATE,
          gas: "0x5208",
          maxFeePerGas: maxFeePerGas,
          maxPriorityFeePerGas: "0xBEBC200", // 0.2GWEI
          to: randomAccount.address,
          data: "0x",
        })
      );
      const postBalance = BigInt(await context.web3.eth.getBalance(alith.address));
      const fee = BigInt(21_000 * maxFeePerGas);
      const expectedPostBalance = preBalance - fee;

      expect(postBalance).to.be.eq(expectedPostBalance);
    });
  },
  "EIP1559"
);

describeDevMoonbeam(
  "Balance transfer - EIP1559 fees",
  (context) => {
    it("should use partial max_priority_fee_per_gas", async function () {
      const randomAccount = generateKeyringPair();
      const preBalance = BigInt(await context.web3.eth.getBalance(alith.address));
      // With this configuration only half of the priority fee will be used, as the max_fee_per_gas
      // is 2GWEI and the base fee is 1GWEI.
      const maxFeePerGas = 1_000_000_000 * 2;

      await context.createBlock(
        createTransaction(context, {
          ...ALITH_TRANSACTION_TEMPLATE,
          gas: "0x5208",
          maxFeePerGas: maxFeePerGas,
          maxPriorityFeePerGas: maxFeePerGas,
          to: randomAccount.address,
          data: "0x",
        })
      );
      const postBalance = BigInt(await context.web3.eth.getBalance(alith.address));
      const fee = BigInt(21_000 * maxFeePerGas);
      const expectedPostBalance = preBalance - fee;

      expect(postBalance).to.be.eq(expectedPostBalance);
    });
  },
  "EIP1559"
);

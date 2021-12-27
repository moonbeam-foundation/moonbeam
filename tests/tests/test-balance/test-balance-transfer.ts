import { expect } from "chai";
import { verifyLatestBlockFees } from "../../util/block";
import { GENESIS_ACCOUNT, GENESIS_ACCOUNT_BALANCE } from "../../util/constants";

import { describeDevMoonbeam, describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import { createTransfer } from "../../util/transactions";
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
        await context.polkadotApi.query.system.account.at(block1Hash, GENESIS_ACCOUNT)
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

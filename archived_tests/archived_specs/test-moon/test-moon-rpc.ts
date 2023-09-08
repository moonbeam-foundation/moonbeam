import "@moonbeam-network/api-augment";
import { describeDevMoonbeam, describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import { expect, use as chaiUse } from "chai";
import chaiAsPromised from "chai-as-promised";
import { createTransaction, DEFAULT_TXN_MAX_BASE_FEE } from "../../util/transactions";
import { ALITH_PRIVATE_KEY, BALTATHAR_ADDRESS } from "../../util/accounts";
chaiUse(chaiAsPromised);

describeDevMoonbeam("Moon RPC Methods - moon_isBlockFinalized ", (context) => {
  it("should return as finalized when true", async function () {
    const blockHash = (await context.createBlock([], { finalize: true })).block.hash;
    const resp = await context.polkadotApi.rpc.moon.isBlockFinalized(blockHash);
    expect(resp.isTrue, "Block finalization status mismatch").to.be.true;
  });

  it("should return as unfinalized when false", async function () {
    const blockHash = (await context.createBlock([], { finalize: false })).block.hash;
    const resp = await context.polkadotApi.rpc.moon.isBlockFinalized(blockHash);
    expect(resp.isFalse, "Block finalization status mismatch").to.be.true;
  });

  it("should return as unfinalized when block not found", async function () {
    const blockHash = "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
    const resp = await context.polkadotApi.rpc.moon.isBlockFinalized(blockHash);
    expect(resp.isFalse, "Block finalization status mismatch").to.be.true;
  });

  it("should return as finalized when new block is true", async function () {
    const blockHash = (await context.createBlock([], { finalize: false })).block.hash;
    await context.createBlock([], { finalize: true });
    const resp = await context.polkadotApi.rpc.moon.isBlockFinalized(blockHash);
    expect(resp.isTrue, "Block finalization status mismatch").to.be.true;
  });

  it("should return as finalized when new block reorg happens", async function () {
    const blockHash = (await context.createBlock([], { finalize: false })).block.hash;
    await context.createBlock([], { finalize: false });
    await context.createBlock([], { finalize: true, parentHash: blockHash });

    const resp = await context.polkadotApi.rpc.moon.isBlockFinalized(blockHash);
    expect(resp.isTrue, "Block finalization status mismatch").to.be.true;
  });
});

describeDevMoonbeamAllEthTxTypes("Moon RPC Methods - moon_isTxFinalized", (context) => {
  it("should return as finalized when true", async function () {
    await context.createBlock(
      createTransaction(context, {
        privateKey: ALITH_PRIVATE_KEY,
        to: BALTATHAR_ADDRESS,
        gas: 12_000_000,
        gasPrice: DEFAULT_TXN_MAX_BASE_FEE,
        value: 1_000_000,
      }),
      { finalize: true }
    );

    const block = await context.web3.eth.getBlock("latest");
    const resp = await context.polkadotApi.rpc.moon.isTxFinalized(block.transactions[0]);
    expect(resp.isTrue, "Transaction finalization status mismatch").to.be.true;
  });

  it("should return as unfinalized when false", async function () {
    await context.createBlock(
      createTransaction(context, {
        privateKey: ALITH_PRIVATE_KEY,
        to: BALTATHAR_ADDRESS,
        gas: 12_000_000,
        gasPrice: DEFAULT_TXN_MAX_BASE_FEE,
        value: 1_000_000,
      }),
      { finalize: false }
    );

    const block = await context.web3.eth.getBlock("latest");
    const resp = await context.polkadotApi.rpc.moon.isTxFinalized(block.transactions[0]);
    expect(resp.isFalse, "Transaction finalization status mismatch").to.be.true;
  });

  it("should return as unfinalized when txn not found", async function () {
    const txnHash = "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
    const resp = await context.polkadotApi.rpc.moon.isTxFinalized(txnHash);
    expect(resp.isFalse, "Transaction finalization status mismatch").to.be.true;
  });

  it("should return as finalized when new block is true", async function () {
    await context.createBlock(
      createTransaction(context, {
        privateKey: ALITH_PRIVATE_KEY,
        to: BALTATHAR_ADDRESS,
        gas: 12_000_000,
        gasPrice: DEFAULT_TXN_MAX_BASE_FEE,
        value: 1_000_000,
      }),
      { finalize: false }
    );

    const block = await context.web3.eth.getBlock("latest");
    await context.createBlock([], { finalize: true });
    const resp = await context.polkadotApi.rpc.moon.isTxFinalized(block.transactions[0]);
    expect(resp.isTrue, "Transaction finalization status mismatch").to.be.true;
  });

  it("should return as finalized when new block reorg happens", async function () {
    const blockHash = (
      await context.createBlock(
        createTransaction(context, {
          privateKey: ALITH_PRIVATE_KEY,
          to: BALTATHAR_ADDRESS,
          gas: 12_000_000,
          gasPrice: DEFAULT_TXN_MAX_BASE_FEE,
          value: 1_000_000,
        }),
        { finalize: false }
      )
    ).block.hash;

    const block = await context.web3.eth.getBlock("latest");
    await context.createBlock([], { finalize: false });
    await context.createBlock([], { finalize: true, parentHash: blockHash });
    const resp = await context.polkadotApi.rpc.moon.isTxFinalized(block.transactions[0]);
    expect(resp.isTrue, "Transaction finalization status mismatch").to.be.true;
  });
});

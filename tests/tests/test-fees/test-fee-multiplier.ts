import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { BN, bnToHex } from "@polkadot/util";
import {
  TREASURY_ACCOUNT,
  MIN_GLMR_STAKING,
  PRECOMPILE_PARACHAIN_STAKING_ADDRESS,
} from "../../util/constants";
import { describeDevMoonbeamAllEthTxTypes, describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createTransfer, sendPrecompileTx } from "../../util/transactions";
import {
  baltathar,
  BALTATHAR_PRIVATE_KEY,
  charleth,
  CHARLETH_PRIVATE_KEY,
} from "../../util/accounts";
import { u128 } from "@polkadot/types";
import { alith } from "../../util/accounts";
import { createContract, createContractExecution } from "../../util/transactions";
import { customWeb3Request } from "../../util/providers";

// Note on the values from 'transactionPayment.nextFeeMultiplier': this storage item is actually a
// FixedU128, which is basically a u128 with an implicit denominator of 10^18. However, this
// denominator is omitted when it is queried through the API, leaving some very large numbers.
//
// To make sense of them, basically remove 18 zeroes (divide by 10^18). This will give you the
// number used internally by transaction-payment for fee calculations.

describeDevMoonbeam("Max Fee Multiplier", (context) => {
  beforeEach("set to max multiplier", async () => {
    const MULTIPLIER_STORAGE_KEY = context.polkadotApi.query.transactionPayment.nextFeeMultiplier
      .key(0)
      .toString();

    const U128_MAX = new BN("340282366920938463463374607431768211455");
    const newMultiplierValue = context.polkadotApi.createType("u128", U128_MAX);

    // set transaction-payment's multiplier to something above max in storage. on the next round,
    // it should enforce its upper bound and reset it.
    await context.polkadotApi.tx.sudo
      .sudo(
        context.polkadotApi.tx.system.setStorage([
          [MULTIPLIER_STORAGE_KEY, bnToHex(newMultiplierValue)],
        ])
      )
      .signAndSend(alith);
    await context.createBlock();
  });

  it("should enforce upper bound", async () => {
    // we set it to u128_max, but the max should have been enforced in on_finalize()
    const multiplier = (
      await context.polkadotApi.query.transactionPayment.nextFeeMultiplier()
    ).toBigInt();
    expect(multiplier).to.equal(100_000_000_000_000_000_000_000n);
  });

  it("should have spendable runtime upgrade", async () => {
    const multiplier = (
      await context.polkadotApi.query.transactionPayment.nextFeeMultiplier()
    ).toBigInt();
    expect(multiplier).to.equal(100_000_000_000_000_000_000_000n);

    const initialBalance = (
      await context.polkadotApi.query.system.account(baltathar.address)
    ).data.free.toBigInt();

    // generate a mock runtime upgrade hex string
    let size = 4194304; // 2MB bytes represented in hex
    let hex = "0x" + "F".repeat(size);

    // send an enactAuthorizedUpgrade. we expect this to fail, but we just want to see that it was
    // included in a block (not rejected) and was charged based on its length
    await context.polkadotApi.tx.parachainSystem.enactAuthorizedUpgrade(hex).signAndSend(baltathar);
    await context.createBlock();

    let afterBalance = (
      await context.polkadotApi.query.system.account(baltathar.address)
    ).data.free.toBigInt();

    // note that this is not really affected by the high multiplier because most of its fee is
    // derived from the length_fee, which is not scaled by the multiplier
    expect(initialBalance - afterBalance).to.equal(9_231_801_265_723_667_008n);
  });

  it("should have spendable fill_block", async () => {
    const multiplier = (
      await context.polkadotApi.query.transactionPayment.nextFeeMultiplier()
    ).toBigInt();
    expect(multiplier).to.equal(100_000_000_000_000_000_000_000n);

    // fill_block will not charge its full amount for us, but we can inspect the initial balance
    // withdraw event to see what it would charge. it is root only and will refund if not called by
    // root, but sudo will also cause a refund.

    let fillAmount = 600_000_000; // equal to 60% Perbill

    const { block, result } = await context.createBlock(
      context.polkadotApi.tx.system.fillBlock(fillAmount)
    );

    // grab the first withdraw event and hope it's the right one...
    const withdrawEvent = result.events.filter(({ event }) => event.method == "Withdraw")[0];
    let amount = (withdrawEvent.event.data as any).amount.toBigInt();
    expect(amount).to.equal(1_500_000_012_598_000_941_192n);
  });

  // similar to tests in test-contract-fibonacci.ts, which implements an Ethereum txn which uses
  // most of the block gas limit. This is done with the fee at its max, however.
  it("fibonacci[370] should be spendable", async function () {
    let blockNumber = (await context.polkadotApi.rpc.chain.getHeader()).number.toBn();
    let baseFeePerGas = BigInt((await context.web3.eth.getBlock(blockNumber)).baseFeePerGas);
    expect(baseFeePerGas).to.equal(125_000_000_000_000n);

    const { contract, rawTx } = await createContract(context, "Fibonacci", {
      gasPrice: "0x" + baseFeePerGas.toString(16),
    });
    const {
      result: { hash: createTxHash },
    } = await context.createBlock(rawTx);

    let receipt = await context.web3.eth.getTransactionReceipt(createTxHash);
    expect(receipt.status).to.be.true;

    // the multiplier (and thereby base_fee) will have decreased very slightly...
    blockNumber = (await context.polkadotApi.rpc.chain.getHeader()).number.toBn();
    baseFeePerGas = BigInt((await context.web3.eth.getBlock(blockNumber)).baseFeePerGas);
    expect(baseFeePerGas).to.equal(124_880_845_878_351n);

    const tx = await createContractExecution(
      context,
      {
        contract,
        contractCall: contract.methods.fib2(370),
      },
      { gasPrice: "0x" + baseFeePerGas.toString(16) }
    );
    let { result } = await context.createBlock(tx);

    receipt = await context.web3.eth.getTransactionReceipt(result.hash);
    expect(receipt.status).to.be.true;

    const successEvent = result.events.filter(({ event }) => event.method == "ExtrinsicSuccess")[0];
    let weight = (successEvent.event.data as any).dispatchInfo.weight.refTime.toBigInt();
    expect(weight).to.equal(4_162_425_000n);

    const withdrawEvents = result.events.filter(({ event }) => event.method == "Withdraw");
    expect(withdrawEvents.length).to.equal(1);
    const withdrawEvent = withdrawEvents[0];
    let amount = (withdrawEvent.event.data as any).amount.toBigInt();
    expect(amount).to.equal(20_828_626_522_358_406_588n);
  });
});

describeDevMoonbeam("Max Fee Multiplier - initial value", (context) => {
  it("should start with genesis value", async () => {
    const initialValue = (
      await context.polkadotApi.query.transactionPayment.nextFeeMultiplier()
    ).toBigInt();
    expect(initialValue).to.equal(8_000_000_000_000_000_000n);
  });
});

import "@moonbeam-network/api-augment/moonbase";
import { expect } from "chai";
import { BN, bnToHex } from "@polkadot/util";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { baltathar, BALTATHAR_ADDRESS, generateKeyringPair } from "../../util/accounts";
import { alith } from "../../util/accounts";
import { createContract, createContractExecution } from "../../util/transactions";
import {
  RawXcmMessage,
  XcmFragment,
  descendOriginFromAddress,
  injectHrmpMessageAndSeal,
} from "../../util/xcm";
import { expectOk } from "../../util/expect";
import { KeyringPair } from "@substrate/txwrapper-core";
import { TARGET_FILL_AMOUNT } from "../../util/constants";

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

    // set transaction-payment's multiplier to something above max in storage. on the next round,
    // it should enforce its upper bound and reset it.
    await context.polkadotApi.tx.sudo
      .sudo(
        context.polkadotApi.tx.system.setStorage([
          [MULTIPLIER_STORAGE_KEY, bnToHex(U128_MAX, { isLe: true, bitLength: 128 })],
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

    const result = await context.ethers.send("eth_gasPrice", []);
    const gasPrice = BigInt(result);
    expect(gasPrice).to.eq(125_000_000_000_000n);
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
      context.polkadotApi.tx.rootTesting.fillBlock(fillAmount)
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
      gasPrice: baseFeePerGas,
    });
    const {
      result: { hash: createTxHash },
    } = await context.createBlock(rawTx);

    let receipt = await context.web3.eth.getTransactionReceipt(createTxHash);
    expect(receipt.status).to.be.true;

    // the multiplier (and thereby base_fee) will have decreased very slightly...
    blockNumber = (await context.polkadotApi.rpc.chain.getHeader()).number.toBn();
    baseFeePerGas = BigInt((await context.web3.eth.getBlock(blockNumber)).baseFeePerGas);
    expect(baseFeePerGas).to.equal(124_880_903_689_844n);

    const tx = await createContractExecution(
      context,
      {
        contract,
        contractCall: contract.methods.fib2(370),
      },
      { gasPrice: baseFeePerGas }
    );
    let { result } = await context.createBlock(tx);

    receipt = await context.web3.eth.getTransactionReceipt(result.hash);
    expect(receipt.status).to.be.true;

    const successEvent = result.events.filter(({ event }) => event.method == "ExtrinsicSuccess")[0];
    let weight = (successEvent.event.data as any).dispatchInfo.weight.refTime.toBigInt();
    expect(weight).to.equal(2_396_800_000n);

    const withdrawEvents = result.events.filter(({ event }) => event.method == "Withdraw");
    expect(withdrawEvents.length).to.equal(1);
    const withdrawEvent = withdrawEvents[0];
    let amount = (withdrawEvent.event.data as any).amount.toBigInt();
    expect(amount).to.equal(11_986_693_540_669_676_340n);
  });
});

describeDevMoonbeam("Min Fee Multiplier", (context) => {
  beforeEach("set to min multiplier", async () => {
    const MULTIPLIER_STORAGE_KEY = context.polkadotApi.query.transactionPayment.nextFeeMultiplier
      .key(0)
      .toString();

    // set transaction-payment's multiplier to something above max in storage. on the next round,
    // it should enforce its upper bound and reset it.
    await context.polkadotApi.tx.sudo
      .sudo(
        context.polkadotApi.tx.system.setStorage([
          [MULTIPLIER_STORAGE_KEY, bnToHex(1n, { isLe: true, bitLength: 128 })],
        ])
      )
      .signAndSend(alith);
    await context.createBlock();
  });

  it("should enforce lower bound", async () => {
    const MULTIPLIER_STORAGE_KEY = context.polkadotApi.query.transactionPayment.nextFeeMultiplier
      .key(0)
      .toString();

    // we set it to u128_max, but the max should have been enforced in on_finalize()
    const multiplier = (
      await context.polkadotApi.query.transactionPayment.nextFeeMultiplier()
    ).toBigInt();
    expect(multiplier).to.equal(100_000_000_000_000_000n);

    const result = await context.ethers.send("eth_gasPrice", []);
    const gasPrice = BigInt(result);
    expect(gasPrice).to.eq(125_000_000n);
  });
});

describeDevMoonbeam("Max Fee Multiplier - initial value", (context) => {
  it("should start with genesis value", async () => {
    const initialValue = (
      await context.polkadotApi.query.transactionPayment.nextFeeMultiplier()
    ).toBigInt();
    expect(initialValue).to.equal(8_000_000_000_000_000_000n);

    const result = await context.ethers.send("eth_gasPrice", []);
    const gasPrice = BigInt(result);
    expect(gasPrice).to.eq(10_000_000_000n);
  });
});

describeDevMoonbeam("Fee Multiplier - XCM Executions", (context) => {
  const startingBn = new BN("2000000000000000000");
  let sendingAddress: string;
  let random: KeyringPair;
  let transferredBalance: bigint;
  let balancesPalletIndex: number;

  before("Suite Setup", async function () {
    const { originAddress, descendOriginAddress } = descendOriginFromAddress(context);
    sendingAddress = originAddress;
    random = generateKeyringPair();
    transferredBalance = 10_000_000_000_000_000_000n;

    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.balances.transfer(descendOriginAddress, transferredBalance * 100n)
      )
    );

    const metadata = await context.polkadotApi.rpc.state.getMetadata();
    balancesPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find((pallet) => {
      return pallet.name === "Balances";
    }).index;
  });

  beforeEach("Reset multiplier", async function () {
    const MULTIPLIER_STORAGE_KEY = context.polkadotApi.query.transactionPayment.nextFeeMultiplier
      .key(0)
      .toString();

    await context.polkadotApi.tx.sudo
      .sudo(
        context.polkadotApi.tx.system.setStorage([
          [MULTIPLIER_STORAGE_KEY, bnToHex(startingBn, { isLe: true, bitLength: 128 })],
        ])
      )
      .signAndSend(alith);
    await context.createBlock();
  });

  it("should decay with no activity", async function () {
    const initialValue = await context.polkadotApi.query.transactionPayment.nextFeeMultiplier();
    await context.createBlock();
    const postValue = await context.polkadotApi.query.transactionPayment.nextFeeMultiplier();
    expect(initialValue.gt(postValue), "Fee Multiplier value not decayed").to.be.true;
  });

  it("should not decay when block size at target amount", async function () {
    const initialValue = await context.polkadotApi.query.transactionPayment.nextFeeMultiplier();
    await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.rootTesting.fillBlock(TARGET_FILL_AMOUNT)
      )
    );
    const postValue = await context.polkadotApi.query.transactionPayment.nextFeeMultiplier();
    expect(initialValue.eq(postValue), "Fee multiplier not static on ideal fill ratio").to.be.true;
  });

  it("should increase when above target fill ratio", async function () {
    const initialValue = await context.polkadotApi.query.transactionPayment.nextFeeMultiplier();
    await context.polkadotApi.tx.balances
      .transfer(BALTATHAR_ADDRESS, 1_000_000_000_000_000_000n)
      .signAndSend(alith, { nonce: -1 });
    await context.polkadotApi.tx.sudo
      .sudo(context.polkadotApi.tx.rootTesting.fillBlock(TARGET_FILL_AMOUNT))
      .signAndSend(alith, { nonce: -1 });
    await context.createBlock();

    const postValue = await context.polkadotApi.query.transactionPayment.nextFeeMultiplier();
    expect(initialValue.lt(postValue), "Fee multiplier not increased when above ideal fill ratio")
      .to.be.true;
  });

  it("should not increase fees with xcm activity", async () => {
    const transferCallEncoded = context.polkadotApi.tx.balances
      .transfer(random.address, transferredBalance / 10n)
      .method.toHex();

    const initialValue = await context.polkadotApi.query.transactionPayment.nextFeeMultiplier();
    const initialBalance = (await context.polkadotApi.query.system.account(random.address)).data
      .free;
    const initialHeight = (
      await context.polkadotApi.rpc.chain.getBlock()
    ).block.header.number.toNumber();

    await context.polkadotApi.tx.sudo
      .sudo(context.polkadotApi.tx.rootTesting.fillBlock(TARGET_FILL_AMOUNT))
      .signAndSend(alith, { nonce: -1 });
    const xcmMessage = new XcmFragment({
      assets: [
        {
          multilocation: {
            parents: 0,
            interior: {
              X1: { PalletInstance: balancesPalletIndex },
            },
          },
          fungible: transferredBalance / 3n,
        },
      ],
      weight_limit: new BN(4000000000),
      descend_origin: sendingAddress,
    })
      .descend_origin()
      .withdraw_asset()
      .buy_execution()
      .push_any({
        Transact: {
          originType: "SovereignAccount",
          requireWeightAtMost: new BN(1000000000),
          call: {
            encoded: transferCallEncoded,
          },
        },
      })
      .as_v2();

    await injectHrmpMessageAndSeal(context, 1, {
      type: "XcmVersionedXcm",
      payload: xcmMessage,
    } as RawXcmMessage);

    const postValue = await context.polkadotApi.query.transactionPayment.nextFeeMultiplier();
    const postBalance = (await context.polkadotApi.query.system.account(random.address)).data.free;
    const postHeight = (
      await context.polkadotApi.rpc.chain.getBlock()
    ).block.header.number.toNumber();

    expect(initialHeight).to.equal(postHeight - 1);
    expect(initialBalance.lt(postBalance), "Expected balances not updated").to.be.true;
    expect(initialValue.eq(postValue), "Fee Multiplier has changed between blocks").to.be.true;
  });

  it("should not increase fees with xcm ETH activity", async () => {
    const amountToTransfer = transferredBalance / 10n;
    const xcmTransactions = [
      {
        V1: {
          gas_limit: 21000,
          fee_payment: {
            Auto: {
              Low: null,
            },
          },
          action: {
            Call: random.address,
          },
          value: amountToTransfer,
          input: [],
          access_list: null,
        },
      },
      {
        V2: {
          gas_limit: 21000,
          action: {
            Call: random.address,
          },
          value: amountToTransfer,
          input: [],
          access_list: null,
        },
      },
    ];
    const transferCallEncodedV1 = context.polkadotApi.tx.ethereumXcm
      .transact(xcmTransactions[0] as any)
      .method.toHex();
    const transferCallEncodedV2 = context.polkadotApi.tx.ethereumXcm
      .transact(xcmTransactions[1] as any)
      .method.toHex();

    const initialValue = await context.polkadotApi.query.transactionPayment.nextFeeMultiplier();
    const initialBalance = (await context.polkadotApi.query.system.account(random.address)).data
      .free;
    const initialHeight = (
      await context.polkadotApi.rpc.chain.getBlock()
    ).block.header.number.toNumber();

    await context.polkadotApi.tx.sudo
      .sudo(context.polkadotApi.tx.rootTesting.fillBlock(TARGET_FILL_AMOUNT))
      .signAndSend(alith, { nonce: -1 });
    const xcmMessage = new XcmFragment({
      assets: [
        {
          multilocation: {
            parents: 0,
            interior: {
              X1: { PalletInstance: balancesPalletIndex },
            },
          },
          fungible: transferredBalance / 3n,
        },
      ],
      weight_limit: new BN(4000000000),
      descend_origin: sendingAddress,
    })
      .descend_origin()
      .withdraw_asset()
      .buy_execution()
      .push_any({
        Transact: {
          originType: "SovereignAccount",
          requireWeightAtMost: new BN(1000000000),
          call: {
            encoded: transferCallEncodedV1,
          },
        },
      })
      .as_v2();

    await injectHrmpMessageAndSeal(context, 1, {
      type: "XcmVersionedXcm",
      payload: xcmMessage,
    } as RawXcmMessage);

    const postValue = await context.polkadotApi.query.transactionPayment.nextFeeMultiplier();
    const postBalance = (await context.polkadotApi.query.system.account(random.address)).data.free;
    const postHeight = (
      await context.polkadotApi.rpc.chain.getBlock()
    ).block.header.number.toNumber();

    expect(initialHeight).to.equal(postHeight - 1);
    expect(initialBalance.lt(postBalance), "Expected balances not updated").to.be.true;
    expect(initialValue.eq(postValue), "Fee Multiplier has changed between blocks").to.be.true;
  });
});

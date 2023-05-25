import "@moonbeam-network/api-augment";

import { BN, u8aToHex } from "@polkadot/util";
import { expect } from "chai";
import { ChaChaRng } from "randchacha";

import {
  mockHrmpChannelExistanceTx,
  injectHrmpMessageAndSeal,
  RawXcmMessage,
  injectHrmpMessage,
  weightMessage,
} from "../../util/xcm";

import { describeDevMoonbeam, DevTestContext } from "../../util/setup-dev-tests";

import type { XcmVersionedXcm, XcmVersionedMultiLocation } from "@polkadot/types/lookup";
import { XcmpMessageFormat } from "@polkadot/types/interfaces";
import { customWeb3Request } from "../../util/providers";

import { expectOk } from "../../util/expect";
import { XcmFragment } from "../../util/xcm";
import { GLMR } from "../../util/constants";
import { blake2AsHex } from "@polkadot/util-crypto";

// enum to mark how xcmp execution went
enum XcmpExecution {
  // it means the xcmp message was executed on_initialization
  InitializationExecutedPassingBarrier,
  // it means the xcmp message failed in the barrier check on_initialization
  InitializationExecutedNotPassingBarrier,
  // it means the xcmp was executed on_idle
  OnIdleExecutedPassingBarrier,
}

// Function to calculate how messages coming from different paras will be executed
export async function calculateShufflingAndExecution(
  context: DevTestContext,
  numParaMsgs: number,
  weightUsePerMessage: bigint,
  totalXcmpWeight: bigint
) {
  // There is a maximum 20 messages that can be processed by a single queue in a block.
  // 10 on initialize and 10 on idle.
  expect(numParaMsgs).to.be.lessThanOrEqual(20);
  // the randomization is as follows
  // for every rand number, we do module number of paras
  // the given index is swaped with that obtained with the
  // random number

  let weightAvailable = 0n;
  let weightUsed = 0n;

  // we want to mimic the randomization process in the queue
  let indices = Array.from(Array(numParaMsgs).keys());
  let shouldItExecute = new Array(numParaMsgs).fill(false);

  const seed = await context.polkadotApi.query.system.parentHash();
  const rng = new ChaChaRng(seed);

  const signedBlock = await context.polkadotApi.rpc.chain.getBlock();
  const apiAt = await context.polkadotApi.at(signedBlock.block.header.hash);
  const queueConfig = (await apiAt.query.xcmpQueue.queueConfig()) as any;
  const decay = queueConfig.weightRestrictDecay.refTime.toBigInt();
  const thresholdWeight = queueConfig.thresholdWeight.refTime.toBigInt();

  let max_message_processed_per_queue = 10;
  for (let i = 0; i < numParaMsgs; i++) {
    let rand = rng.nextU32();
    let j = rand % numParaMsgs;
    [indices[i], indices[j]] = [indices[j], indices[i]];

    // mimics the decay algorithm
    if (totalXcmpWeight - weightUsed > thresholdWeight && max_message_processed_per_queue > 0) {
      if (weightAvailable != totalXcmpWeight) {
        weightAvailable += (totalXcmpWeight - weightAvailable) / (decay + 1n);
        if (weightAvailable + thresholdWeight > totalXcmpWeight) {
          weightAvailable = totalXcmpWeight;
        }
      }
      let weight_remaining = weightAvailable - weightUsed;

      if (weight_remaining < weightUsePerMessage) {
        shouldItExecute[i] = XcmpExecution.InitializationExecutedNotPassingBarrier;
      } else {
        shouldItExecute[i] = XcmpExecution.InitializationExecutedPassingBarrier;
        weightUsed += weightUsePerMessage;
      }
      max_message_processed_per_queue--;
    } else {
      // we know this will execute on idle
      shouldItExecute[i] = XcmpExecution.OnIdleExecutedPassingBarrier;
    }
  }

  return [indices, shouldItExecute];
}

describeDevMoonbeam("Mock XCMP - test XCMP execution", (context) => {
  it("Should test that XCMP is executed randomized and until exhausted", async function () {
    this.timeout(120000);
    const metadata = await context.polkadotApi.rpc.state.getMetadata();
    const balancesPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find(
      (pallet) => pallet.name === "Balances"
    ).index;

    // lets work with restrict decay 0 for now
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.sudo.sudo(
          context.polkadotApi.tx.xcmpQueue.updateWeightRestrictDecay({
            refTime: 0,
            proofSize: 0,
          } as any)
        )
      )
    );

    const numParaMsgs = 20;
    // let's target half of then being executed

    // xcmp reserved is BLOCK/4
    const totalXcmpWeight =
      context.polkadotApi.consts.system.blockWeights.maxBlock.refTime.toBigInt() / BigInt(4);

    // we want half of numParaMsgs to be executed. That give us how much each message weights
    const weightPerMessage = (totalXcmpWeight * BigInt(2)) / BigInt(numParaMsgs);

    const config = {
      assets: [
        {
          multilocation: {
            parents: 0,
            interior: {
              X1: { PalletInstance: balancesPalletIndex },
            },
          },
          fungible: 1_000_000_000_000_000n,
        },
      ],
    };

    // How much does the withdraw weight?
    const withdrawWeight = await weightMessage(
      context,
      context.polkadotApi.createType(
        "XcmVersionedXcm",
        new XcmFragment(config).withdraw_asset().as_v2()
      )
    );

    // How much does the buyExecution weight?
    const buyExecutionWeight = await weightMessage(
      context,
      context.polkadotApi.createType(
        "XcmVersionedXcm",
        new XcmFragment(config).buy_execution().as_v2()
      )
    );

    // How much does a base Transact weight?
    const transactBaseWeight = await weightMessage(
      context,
      context.polkadotApi.createType(
        "XcmVersionedXcm",
        new XcmFragment(config)
          .push_any({
            Transact: {
              originType: "SovereignAccount",
              requireWeightAtMost: new BN(0),
              call: {
                encoded: "0x11",
              },
            },
          })
          .as_v2()
      )
    );

    // Now we need to construct the message. This needs to:
    // - pass barrier (withdraw + buyExecution + n*unLimitedbuyExecution)
    // - does not fail, so all weight is counted
    // we know at least 2 instructions are needed per message (withdrawAsset + buyExecution)
    // We will append a custom and single Transact, to match the weight needed

    // In this case we want to never reach the thresholdLimit, to make sure we execute every
    // single messages

    const requireWeightAtMostParemeter =
      weightPerMessage - withdrawWeight - buyExecutionWeight - transactBaseWeight;

    const xcmMessage = new XcmFragment(config)
      .withdraw_asset()
      .buy_execution(0)
      .push_any({
        Transact: {
          originType: "SovereignAccount",
          requireWeightAtMost: requireWeightAtMostParemeter,
          call: {
            encoded: "0x11",
          },
        },
      })
      .as_v2();

    // The way we will prove that the message executed is checking balances.
    // For that, WithdrawAsset needs to work. The only way for this to work
    // is to fund each sovereign account
    for (let i = 0; i < numParaMsgs; i++) {
      const paraId = context.polkadotApi.createType("ParaId", i + 1) as any;
      const sovereignAddress = u8aToHex(
        new Uint8Array([...new TextEncoder().encode("sibl"), ...paraId.toU8a()])
      ).padEnd(42, "0");

      // We first fund each sovereign account with 1 GLMR
      // we will only withdraw 1, so no problem on this
      await expectOk(
        context.createBlock(context.polkadotApi.tx.balances.transfer(sovereignAddress, 1n * GLMR))
      );
    }

    // now we start injecting messages
    // one per para
    for (let i = 0; i < numParaMsgs; i++) {
      await injectHrmpMessage(context, i + 1, {
        type: "XcmVersionedXcm",
        payload: xcmMessage,
      } as RawXcmMessage);
    }

    await context.createBlock();

    // withdrawAsset + BuyExecution + Transact
    const weightUsePerMessage =
      requireWeightAtMostParemeter + transactBaseWeight + buyExecutionWeight + withdrawWeight;

    const result = await calculateShufflingAndExecution(
      context,
      numParaMsgs,
      weightUsePerMessage,
      totalXcmpWeight
    );

    const indices = result[0];
    const shouldItExecute = result[1];

    // assert we dont have on_idle execution
    expect(shouldItExecute.indexOf(XcmpExecution.InitializationExecutedPassingBarrier) > -1).to.be
      .true;
    expect(shouldItExecute.indexOf(XcmpExecution.OnIdleExecutedPassingBarrier) > -1).to.be.true;

    // check balances
    for (let i = 0; i < numParaMsgs; i++) {
      // we need to check the randomization works. We have the shuffleing
      // and the amount executed, we need to make sure the balances
      // corresponding to the first executedCount shuffled indices
      // has one less unit of token
      const paraId = context.polkadotApi.createType("ParaId", i + 1) as any;
      const sovereignAddress = u8aToHex(
        new Uint8Array([...new TextEncoder().encode("sibl"), ...paraId.toU8a()])
      ).padEnd(42, "0");

      let balance = await context.polkadotApi.query.system.account(sovereignAddress);

      expect(balance.data.free.toBigInt()).to.eq(1n * GLMR - 1_000_000_000_000_000n);
    }
  });
});

describeDevMoonbeam("Mock XCMP - test XCMP execution", (context) => {
  it("Should test XCMP with decay randomized until exhausted, then Onidle", async function () {
    this.timeout(120000);

    const metadata = await context.polkadotApi.rpc.state.getMetadata();
    const balancesPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find(
      (pallet) => pallet.name === "Balances"
    ).index;

    const numParaMsgs = 20;
    // let's target half of then being executed

    // xcmp reserved is BLOCK/4
    const totalXcmpWeight =
      context.polkadotApi.consts.system.blockWeights.maxBlock.refTime.toBigInt() / BigInt(4);

    // we want half of numParaMsgs to be executed. That give us how much each message weights
    const weightPerMessage = (totalXcmpWeight * BigInt(2)) / BigInt(numParaMsgs);

    const config = {
      assets: [
        {
          multilocation: {
            parents: 0,
            interior: {
              X1: { PalletInstance: balancesPalletIndex },
            },
          },
          fungible: 1_000_000_000_000_000n,
        },
      ],
    };

    // How much does the withdraw weight?
    const withdrawWeight = await weightMessage(
      context,
      context.polkadotApi.createType(
        "XcmVersionedXcm",
        new XcmFragment(config).withdraw_asset().as_v2()
      )
    );

    // How much does the buyExecution weight?
    const buyExecutionWeight = await weightMessage(
      context,
      context.polkadotApi.createType(
        "XcmVersionedXcm",
        new XcmFragment(config).buy_execution().as_v2()
      )
    );

    // How much does the transact weight with 0 required?
    const transactBaseWeight = await weightMessage(
      context,
      context.polkadotApi.createType(
        "XcmVersionedXcm",
        new XcmFragment(config)
          .push_any({
            Transact: {
              originType: "SovereignAccount",
              requireWeightAtMost: new BN(0),
              call: {
                encoded: 0x01,
              },
            },
          })
          .as_v2()
      )
    );

    let requireWeightAtMostParemeter =
      weightPerMessage - withdrawWeight - buyExecutionWeight - transactBaseWeight;

    const xcmMessage = new XcmFragment(config)
      .withdraw_asset()
      .buy_execution(0)
      // Does not reallly matter, wont be executed, we want it to fail
      .push_any({
        Transact: {
          originType: "SovereignAccount",
          requireWeightAtMost: new BN(requireWeightAtMostParemeter.toString()),
          call: {
            encoded: 0x01,
          },
        },
      })
      .as_v2();

    // We want these isntructions to fail in Transact. That means
    // WithdrawAsset needs to work. The only way for this to work
    // is to fund each sovereign account
    for (let i = 0; i < numParaMsgs; i++) {
      const paraId = context.polkadotApi.createType("ParaId", i + 1) as any;
      const sovereignAddress = u8aToHex(
        new Uint8Array([...new TextEncoder().encode("sibl"), ...paraId.toU8a()])
      ).padEnd(42, "0");

      // We first fund each sovereign account with 100
      // we will only withdraw 1, so no problem on this
      await expectOk(
        context.createBlock(context.polkadotApi.tx.balances.transfer(sovereignAddress, 1n * GLMR))
      );
    }

    // now we start injecting messages
    // one per para
    for (let i = 0; i < numParaMsgs; i++) {
      await injectHrmpMessage(context, i + 1, {
        type: "XcmVersionedXcm",
        payload: xcmMessage,
      } as RawXcmMessage);
    }

    await context.createBlock();

    // in this case, we have some that will execute on_initialize
    // some that will fail the execution
    // and some that will execute on_idle
    const result = await calculateShufflingAndExecution(
      context,
      numParaMsgs,
      weightPerMessage,
      totalXcmpWeight
    );

    const indices = result[0];
    const shouldItExecute = result[1];

    // assert we have all kinds of execution
    expect(shouldItExecute.indexOf(XcmpExecution.InitializationExecutedPassingBarrier) > -1).to.be
      .true;
    expect(shouldItExecute.indexOf(XcmpExecution.InitializationExecutedNotPassingBarrier) > -1).to
      .be.true;
    expect(shouldItExecute.indexOf(XcmpExecution.OnIdleExecutedPassingBarrier) > -1).to.be.true;

    // check balances
    for (let i = 0; i < numParaMsgs; i++) {
      // we need to check the randomization works. We have the shuffleing
      // and the amount executed, we need to make sure the balances
      // corresponding to the first executedCount shuffled indices
      // has one less unit of token
      const paraId = context.polkadotApi.createType("ParaId", i + 1) as any;
      const sovereignAddress = u8aToHex(
        new Uint8Array([...new TextEncoder().encode("sibl"), ...paraId.toU8a()])
      ).padEnd(42, "0");

      const balance = await context.polkadotApi.query.system.account(sovereignAddress);

      if (
        shouldItExecute[indices.indexOf(i)] == XcmpExecution.InitializationExecutedPassingBarrier ||
        shouldItExecute[indices.indexOf(i)] == XcmpExecution.OnIdleExecutedPassingBarrier
      ) {
        expect(balance.data.free.toBigInt()).to.eq(1n * GLMR - 1_000_000_000_000_000n);
      } else {
        expect(balance.data.free.toBigInt()).to.eq(1n * GLMR);
      }
    }
  });
});

describeDevMoonbeam("Mock XCMP - test XCMP execution", (context) => {
  it("Should test for two XCMP insertions for same para, the last is executed", async function () {
    const metadata = await context.polkadotApi.rpc.state.getMetadata();
    const balancesPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find(
      (pallet) => pallet.name === "Balances"
    ).index;

    const xcmMessageNotExecuted = new XcmFragment({
      assets: [
        {
          multilocation: {
            parents: 0,
            interior: {
              X1: { PalletInstance: balancesPalletIndex },
            },
          },
          fungible: 1n,
        },
      ],
      weight_limit: new BN(20000000000),
    })
      .withdraw_asset()
      .buy_execution()
      .as_v2();

    const xcmMessageExecuted = new XcmFragment({
      assets: [
        {
          multilocation: {
            parents: 0,
            interior: {
              X1: { PalletInstance: balancesPalletIndex },
            },
          },
          fungible: 2n,
        },
      ],
      weight_limit: new BN(20000000000),
    })
      .withdraw_asset()
      .buy_execution()
      .as_v2();

    const paraId = context.polkadotApi.createType("ParaId", 1) as any;
    const sovereignAddress = u8aToHex(
      new Uint8Array([...new TextEncoder().encode("sibl"), ...paraId.toU8a()])
    ).padEnd(42, "0");

    // We first fund each sovereign account with 100
    await expectOk(
      context.createBlock(context.polkadotApi.tx.balances.transfer(sovereignAddress, 100n))
    );

    // now we start injecting messages
    // two for para 1
    await injectHrmpMessage(context, 1, {
      type: "XcmVersionedXcm",
      payload: xcmMessageNotExecuted,
    } as RawXcmMessage);

    await injectHrmpMessage(context, 1, {
      type: "XcmVersionedXcm",
      payload: xcmMessageExecuted,
    } as RawXcmMessage);

    await context.createBlock();

    // The balance of the sovereign account should be 98, as the first message does not executed
    const balance = await context.polkadotApi.query.system.account(sovereignAddress);
    expect(balance.data.free.toBigInt()).to.eq(98n);
  });
});

describeDevMoonbeam("Mock XCMP - test XCMP execution", (context) => {
  it("Should test for three XCMP insertions, the channel gests suspended", async function () {
    const metadata = await context.polkadotApi.rpc.state.getMetadata();
    const balancesPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find(
      (pallet) => pallet.name === "Balances"
    ).index;

    const xcmMessageNotExecuted = new XcmFragment({
      assets: [
        {
          multilocation: {
            parents: 0,
            interior: {
              X1: { PalletInstance: balancesPalletIndex },
            },
          },
          fungible: 1n,
        },
      ],
      weight_limit: new BN(20000000000),
    })
      .withdraw_asset()
      .buy_execution()
      .as_v2();

    const paraId = context.polkadotApi.createType("ParaId", 1) as any;
    const sovereignAddress = u8aToHex(
      new Uint8Array([...new TextEncoder().encode("sibl"), ...paraId.toU8a()])
    ).padEnd(42, "0");

    // We first fund each sovereign account with 100
    // we will only withdraw 1, so no problem on this
    await expectOk(
      context.createBlock(context.polkadotApi.tx.balances.transfer(sovereignAddress, 100n))
    );

    const queueConfig = (await context.polkadotApi.query.xcmpQueue.queueConfig()) as any;
    const suspendThreshold = queueConfig.suspendThreshold.toNumber();

    // now we start injecting messages
    for (let i = 0; i < suspendThreshold + 1; i++) {
      await injectHrmpMessage(context, 1, {
        type: "XcmVersionedXcm",
        payload: xcmMessageNotExecuted,
      } as RawXcmMessage);
    }

    const result = await context.createBlock().catch((e) => e);
    expect(result.toString()).to.eq(
      "RpcError: 20000: Error at calling runtime api: " +
        "Execution failed: Runtime panicked: assertion failed: mid <= self.len()"
    );

    // The balance of the sovereign account should be 100, as none of the messages got executed
    const balance = await context.polkadotApi.query.system.account(sovereignAddress);
    expect(balance.data.free.toBigInt()).to.eq(100n);
  });
});

describeDevMoonbeam("Mock XCM - receive horizontal suspend", (context) => {
  const suspendedPara = 2023;
  before("Should receive a suspend channel", async function () {
    // Send an XCM and create block to execute it
    await injectHrmpMessageAndSeal(context, suspendedPara, {
      type: "u8",
      payload: 0,
      format: "Signals",
    } as RawXcmMessage);

    // assert channel with para 2023 is suspended
    const status = await context.polkadotApi.query.xcmpQueue.outboundXcmpStatus();
    expect(status[0].state.isSuspended).to.be.true;
  });

  it("should push messages, and enqueue them in xcmp outbound queue", async function () {
    // TARGET 100 messages
    // We want to check there is no visible limit on these

    // Fragments are grouped together and stored in a message
    // It is this message that we are going to store
    // The easiest way to test it create a single message every block,
    // with no other messages to append

    // We can create these with sudo
    // The simplest message should do it
    const xcmMessage = {
      V2: [{ ClearOrigin: null as any }],
    };

    const messageToSend: XcmVersionedXcm = context.polkadotApi.createType(
      "XcmVersionedXcm",
      xcmMessage
    ) as any;

    const destination = {
      V3: {
        parents: 1,
        interior: { X1: { Parachain: suspendedPara } },
      },
    };

    const versionedMult: XcmVersionedMultiLocation = context.polkadotApi.createType(
      "XcmVersionedMultiLocation",
      destination
    ) as any;

    // We also need to trick parachain-system to pretend there exists
    // an open channel with para id 2023.
    // For channel params, we set the default in all of them except for the maxMessageSize
    // We select MaxMessageSize = 4 because ClearOrigin involves 4 bytes
    // This makes sure that each message is enqued in a different page
    const paraHrmpMockerTx = mockHrmpChannelExistanceTx(context, suspendedPara, 8, 8192, 4);

    // Test for numMessages
    const numMessages = 100;

    for (let i = 0; i < numMessages; i++) {
      await expectOk(
        context.createBlock(
          context.polkadotApi.tx.sudo.sudo(
            context.polkadotApi.tx.utility.batchAll([
              paraHrmpMockerTx,
              context.polkadotApi.tx.polkadotXcm.send(versionedMult, messageToSend),
            ])
          )
        )
      );
    }

    // expect that queued messages is equal to the number of sent messages
    const queuedMessages = await context.polkadotApi.query.xcmpQueue.outboundXcmpMessages.entries();
    expect(queuedMessages).to.have.lengthOf(numMessages);
  });
});

describeDevMoonbeam("Mock XCMP - test XCMP execution", (context) => {
  it("Should test that we receive an event per fragment, not per message", async function () {
    const metadata = await context.polkadotApi.rpc.state.getMetadata();
    const balancesPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find(
      (pallet) => pallet.name === "Balances"
    ).index;

    const sendingParaId = context.polkadotApi.createType("ParaId", 2000) as any;
    const sovereignAddress = u8aToHex(
      new Uint8Array([...new TextEncoder().encode("sibl"), ...sendingParaId.toU8a()])
    ).padEnd(42, "0");

    await expectOk(
      context.createBlock(context.polkadotApi.tx.balances.transfer(sovereignAddress, 1n * GLMR))
    );

    // we will prove we get two different events with xcmp.queue
    const xcmFirstFragment = new XcmFragment({
      assets: [
        {
          multilocation: {
            parents: 0,
            interior: {
              X1: { PalletInstance: balancesPalletIndex },
            },
          },
          fungible: 1_000_000_000_000_000n,
        },
      ],
      weight_limit: new BN(1_000_000_000),
    })
      .withdraw_asset()
      .buy_execution()
      .as_v2();

    const xcmSecondFragment = new XcmFragment({
      assets: [
        {
          multilocation: {
            parents: 0,
            interior: {
              X1: { PalletInstance: balancesPalletIndex },
            },
          },
          fungible: 2_000_000_000_000_000n,
        },
      ],
      weight_limit: new BN(2_000_000_000),
    })
      .withdraw_asset()
      .buy_execution()
      .as_v2();

    // In order to insert two fragments in one msg, we need to manually build the message
    const firstEncodedFragment: XcmVersionedXcm = context.polkadotApi.createType(
      "XcmVersionedXcm",
      xcmFirstFragment
    ) as any;

    const secondEncodedFragment: XcmVersionedXcm = context.polkadotApi.createType(
      "XcmVersionedXcm",
      xcmSecondFragment
    ) as any;

    // We first fund each sovereign account with 100
    // we will only withdraw 1, so no problem on this
    await expectOk(
      context.createBlock(context.polkadotApi.tx.balances.transfer(sovereignAddress, 1n * GLMR))
    );

    const xcmpFormat: XcmpMessageFormat = context.polkadotApi.createType(
      "XcmpMessageFormat",
      "ConcatenatedVersionedXcm"
    ) as any;

    await customWeb3Request(context.web3, "xcm_injectHrmpMessage", [
      2000,
      [...xcmpFormat.toU8a(), ...firstEncodedFragment.toU8a(), ...secondEncodedFragment.toU8a()],
    ]);
    await context.createBlock();

    const records = (await context.polkadotApi.query.system.events()) as any;
    const events = records.filter(
      ({ event }) => event.section == "xcmpQueue" && event.method == "Success"
    );
    expect(events).to.have.lengthOf(2);
    expect(events[0].event.data[0].toString()).to.be.eq(
      blake2AsHex(firstEncodedFragment.toU8a()).toString()
    );
    expect(events[1].event.data[0].toString()).to.be.eq(
      blake2AsHex(secondEncodedFragment.toU8a()).toString()
    );
  });
});

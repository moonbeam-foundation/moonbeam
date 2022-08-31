import "@moonbeam-network/api-augment";

import { BN, u8aToHex } from "@polkadot/util";
import { expect } from "chai";
import { ChaChaRng } from "randchacha";

import {
  mockHrmpChannelExistanceTx,
  injectHrmpMessageAndSeal,
  RawXcmMessage,
  injectHrmpMessage,
} from "../../util/xcm";

import { describeDevMoonbeam, DevTestContext } from "../../util/setup-dev-tests";

import type { XcmVersionedXcm, XcmVersionedMultiLocation } from "@polkadot/types/lookup";

import { expectOk } from "../../util/expect";
import { XcmFragment } from "../../util/xcm";

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
  // the randomization is as follows
  // for every rand number, we do module number of paras
  // the given index is swaped with that obtained with the
  // random number

  let weightAvailable = 0n;
  let weightUsed = 0n;

  // we want to mimic the randomization process in the queue
  let indices = Array.from(Array(numParaMsgs).keys());
  let shouldItExecute = new Array(numParaMsgs).fill(false);

  let seed = await context.polkadotApi.query.system.parentHash();
  let rng = new ChaChaRng(seed);

  const signedBlock = await context.polkadotApi.rpc.chain.getBlock();
  const apiAt = await context.polkadotApi.at(signedBlock.block.header.hash);
  const queueConfig = (await apiAt.query.xcmpQueue.queueConfig()) as any;
  let decay = queueConfig.weightRestrictDecay.toBigInt();
  let thresholdWeight = queueConfig.thresholdWeight.toBigInt();

  for (let i = 0; i < numParaMsgs; i++) {
    let rand = rng.nextU32();
    let j = rand % numParaMsgs;
    [indices[i], indices[j]] = [indices[j], indices[i]];

    // mimics the decay algorithm
    if (totalXcmpWeight - weightUsed > thresholdWeight) {
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
          context.polkadotApi.tx.xcmpQueue.updateWeightRestrictDecay(0)
        )
      )
    );

    const numParaMsgs = 50;
    // let's target half of then being executed

    // xcmp reserved is BLOCK/4
    const totalXcmpWeight =
      context.polkadotApi.consts.system.blockWeights.maxBlock.toBigInt() / BigInt(4);

    // we want half of numParaMsgs to be executed. That give us how much each message weights
    const weightPerMessage = (totalXcmpWeight * BigInt(2)) / BigInt(numParaMsgs);

    const weightPerXcmInst = 200_000_000n;
    // Now we need to construct the message. This needs to:
    // - pass barrier (withdraw + clearOrigin*n buyExecution)
    // - fail in buyExecution, so that the previous instruction weights are counted
    // we know at least 2 instructions are needed per message (withdrawAsset + buyExecution)
    // how many clearOrigins do we need to append?

    // In this case we want to never reach the thresholdLimit, to make sure we execute every
    // single messages.
    const clearOriginsPerMessage = (weightPerMessage - weightPerXcmInst) / weightPerXcmInst;

    const xcmMessage = new XcmFragment({
      fees: {
        multilocation: [
          {
            parents: 0,
            interior: {
              X1: { PalletInstance: balancesPalletIndex },
            },
          },
        ],
        fungible: 1n,
      },
      weight_limit: new BN(20000000000),
    })
      .withdraw_asset()
      .clear_origin(clearOriginsPerMessage)
      .buy_execution()
      .as_v2();

    // We want these isntructions to fail in BuyExecution. That means
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
        context.createBlock(context.polkadotApi.tx.balances.transfer(sovereignAddress, 100n))
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

    // all the withdraws + clear origins `buyExecution
    const weightUsePerMessage = (clearOriginsPerMessage + 2n) * weightPerXcmInst;

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
    expect(shouldItExecute.indexOf(XcmpExecution.InitializationExecutedNotPassingBarrier) > -1).to
      .be.true;
    expect(shouldItExecute.indexOf(XcmpExecution.OnIdleExecutedPassingBarrier) > -1).to.be.false;

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

      if (
        shouldItExecute[indices.indexOf(i)] == XcmpExecution.InitializationExecutedPassingBarrier ||
        shouldItExecute[indices.indexOf(i)] == XcmpExecution.OnIdleExecutedPassingBarrier
      ) {
        expect(balance.data.free.toBigInt()).to.eq(99n);
      } else {
        expect(balance.data.free.toBigInt()).to.eq(100n);
      }
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

    const numParaMsgs = 50;
    // let's target half of then being executed

    // xcmp reserved is BLOCK/4
    const totalXcmpWeight =
      context.polkadotApi.consts.system.blockWeights.maxBlock.toBigInt() / BigInt(4);

    // we want half of numParaMsgs to be executed. That give us how much each message weights
    const weightPerMessage = (totalXcmpWeight * BigInt(2)) / BigInt(numParaMsgs);

    const weightPerXcmInst = 200_000_000n;
    // Now we need to construct the message. This needs to:
    // - pass barrier (withdraw + clearOrigin*n buyExecution)
    // - fail in buyExecution, so that the previous instruction weights are counted
    // we know at least 2 instructions are needed per message (withdrawAsset + buyExecution)
    // how many clearOrigins do we need to append?

    // we will bias this number. The reason is we want to test the decay, and therefore we need
    // an unbalanced number of messages executed. We specifically need that at some point
    // we get out of the loop of the execution (we reach the threshold limit), to then
    // go on idle

    // for that reason, we multiply times 2
    const clearOriginsPerMessage = (weightPerMessage - weightPerXcmInst * 2n) / weightPerXcmInst;

    const xcmMessage = new XcmFragment({
      fees: {
        multilocation: [
          {
            parents: 0,
            interior: {
              X1: { PalletInstance: balancesPalletIndex },
            },
          },
        ],
        fungible: 1n,
      },
      weight_limit: new BN(20000000000),
    })
      .withdraw_asset()
      .clear_origin(clearOriginsPerMessage)
      .buy_execution()
      .as_v2();

    // We want these isntructions to fail in BuyExecution. That means
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
        context.createBlock(context.polkadotApi.tx.balances.transfer(sovereignAddress, 100n))
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

    // all the withdraws + clear origins `buyExecution
    const weightUsePerMessage = (clearOriginsPerMessage + 2n) * weightPerXcmInst;

    // in this case, we have some that will execute on_initialize
    // some that will fail the execution
    // and some that will execute on_idle
    const result = await calculateShufflingAndExecution(
      context,
      numParaMsgs,
      weightUsePerMessage,
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
        expect(balance.data.free.toBigInt()).to.eq(99n);
      } else {
        expect(balance.data.free.toBigInt()).to.eq(100n);
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
      fees: {
        multilocation: [
          {
            parents: 0,
            interior: {
              X1: { PalletInstance: balancesPalletIndex },
            },
          },
        ],
        fungible: 1n,
      },
      weight_limit: new BN(20000000000),
    })
      .withdraw_asset()
      .buy_execution()
      .as_v2();

    const xcmMessageExecuted = new XcmFragment({
      fees: {
        multilocation: [
          {
            parents: 0,
            interior: {
              X1: { PalletInstance: balancesPalletIndex },
            },
          },
        ],
        fungible: 2n,
      },
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
      fees: {
        multilocation: [
          {
            parents: 0,
            interior: {
              X1: { PalletInstance: balancesPalletIndex },
            },
          },
        ],
        fungible: 1n,
      },
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
      V1: {
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

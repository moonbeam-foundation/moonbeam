import "@moonbeam-network/api-augment";

import { XcmpMessageFormat } from "@polkadot/types/interfaces";
import { BN, u8aToHex } from "@polkadot/util";
import { expect } from "chai";
import { ChaChaRng } from "randchacha";

import { customWeb3Request } from "../../util/providers";
import { mockHrmpChannelExistanceTx } from "../../util/xcm";

import { describeDevMoonbeam, DevTestContext } from "../../util/setup-dev-tests";

import type { XcmVersionedXcm, XcmVersionedMultiLocation } from "@polkadot/types/lookup";

import { expectOk } from "../../util/expect";

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

    const weightPerXcmInst = 100_000_000n;
    // Now we need to construct the message. This needs to:
    // - pass barrier (withdraw + clearOrigin*n buyExecution)
    // - fail in buyExecution, so that the previous instruction weights are counted
    // we know at least 2 instructions are needed per message (withdrawAsset + buyExecution)
    // how many clearOrigins do we need to append?

    // In this case we want to never reach the thresholdLimit, to make sure we execute every
    // single messages.
    const clearOriginsPerMessage = (weightPerMessage - weightPerXcmInst) / weightPerXcmInst;

    const instructions = [
      {
        WithdrawAsset: [
          {
            id: {
              Concrete: {
                parents: 0,
                interior: {
                  X1: { PalletInstance: balancesPalletIndex },
                },
              },
            },
            fun: { Fungible: 1 },
          },
        ],
      },
      ...Array(Number(clearOriginsPerMessage)).fill({
        ClearOrigin: null,
      }),
      {
        BuyExecution: {
          fees: {
            id: {
              Concrete: {
                parents: 0,
                interior: {
                  X1: { PalletInstance: balancesPalletIndex },
                },
              },
            },
            fun: { Fungible: 1 },
          },
          weightLimit: { Limited: new BN(20000000000) },
        },
      },
    ];

    const xcmMessage = {
      V2: instructions,
    };
    const xcmpFormat: XcmpMessageFormat = context.polkadotApi.createType(
      "XcmpMessageFormat",
      "ConcatenatedVersionedXcm"
    ) as any;
    const receivedMessage: XcmVersionedXcm = context.polkadotApi.createType(
      "XcmVersionedXcm",
      xcmMessage
    ) as any;

    const totalMessage = [...xcmpFormat.toU8a(), ...receivedMessage.toU8a()];

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
      await customWeb3Request(context.web3, "xcm_injectHrmpMessage", [i + 1, totalMessage]);
    }

    await context.createBlock();

    // all the withdraws + clear origins `buyExecution
    const weightUsePerMessage = (clearOriginsPerMessage + 2n) * 100_000_000n;

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

    const weightPerXcmInst = 100_000_000n;
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

    const instructions = [
      {
        WithdrawAsset: [
          {
            id: {
              Concrete: {
                parents: 0,
                interior: {
                  X1: { PalletInstance: balancesPalletIndex },
                },
              },
            },
            fun: { Fungible: 1 },
          },
        ],
      },
      ...Array(Number(clearOriginsPerMessage)).fill({
        ClearOrigin: null,
      }),
      {
        BuyExecution: {
          fees: {
            id: {
              Concrete: {
                parents: 0,
                interior: {
                  X1: { PalletInstance: balancesPalletIndex },
                },
              },
            },
            fun: { Fungible: 1 },
          },
          weightLimit: { Limited: new BN(20000000000) },
        },
      },
    ];

    let xcmMessage = {
      V2: instructions,
    };
    const xcmpFormat: XcmpMessageFormat = context.polkadotApi.createType(
      "XcmpMessageFormat",
      "ConcatenatedVersionedXcm"
    ) as any;
    const receivedMessage: XcmVersionedXcm = context.polkadotApi.createType(
      "XcmVersionedXcm",
      xcmMessage
    ) as any;

    const totalMessage = [...xcmpFormat.toU8a(), ...receivedMessage.toU8a()];

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
      await customWeb3Request(context.web3, "xcm_injectHrmpMessage", [i + 1, totalMessage]);
    }

    await context.createBlock();

    // all the withdraws + clear origins `buyExecution
    const weightUsePerMessage = (clearOriginsPerMessage + 2n) * 100_000_000n;

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

    const instructionsNotExecuted = [
      {
        WithdrawAsset: [
          {
            id: {
              Concrete: {
                parents: 0,
                interior: {
                  X1: { PalletInstance: balancesPalletIndex },
                },
              },
            },
            fun: { Fungible: 1 },
          },
        ],
      },
      {
        BuyExecution: {
          fees: {
            id: {
              Concrete: {
                parents: 0,
                interior: {
                  X1: { PalletInstance: balancesPalletIndex },
                },
              },
            },
            fun: { Fungible: 1 },
          },
          weightLimit: { Limited: new BN(20000000000) },
        },
      },
    ];

    const instructionsExecuted = [
      {
        WithdrawAsset: [
          {
            id: {
              Concrete: {
                parents: 0,
                interior: {
                  X1: { PalletInstance: balancesPalletIndex },
                },
              },
            },
            fun: { Fungible: 2 },
          },
        ],
      },
      {
        BuyExecution: {
          fees: {
            id: {
              Concrete: {
                parents: 0,
                interior: {
                  X1: { PalletInstance: balancesPalletIndex },
                },
              },
            },
            fun: { Fungible: 2 },
          },
          weightLimit: { Limited: new BN(20000000000) },
        },
      },
    ];

    const xcmMessageNotExecuted = {
      V2: instructionsNotExecuted,
    };
    const xcmMessageExecuted = {
      V2: instructionsExecuted,
    };
    const xcmpFormat: XcmpMessageFormat = context.polkadotApi.createType(
      "XcmpMessageFormat",
      "ConcatenatedVersionedXcm"
    ) as any;
    const receivedMessageNotExecuted: XcmVersionedXcm = context.polkadotApi.createType(
      "XcmVersionedXcm",
      xcmMessageNotExecuted
    ) as any;

    const receivedMessageExecuted: XcmVersionedXcm = context.polkadotApi.createType(
      "XcmVersionedXcm",
      xcmMessageExecuted
    ) as any;

    const totalMessageNotExecuted = [...xcmpFormat.toU8a(), ...receivedMessageNotExecuted.toU8a()];
    const totalMessageExecuted = [...xcmpFormat.toU8a(), ...receivedMessageExecuted.toU8a()];

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
    await customWeb3Request(context.web3, "xcm_injectHrmpMessage", [1, totalMessageNotExecuted]);
    await customWeb3Request(context.web3, "xcm_injectHrmpMessage", [1, totalMessageExecuted]);

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

    const instructionsNotExecuted = [
      {
        WithdrawAsset: [
          {
            id: {
              Concrete: {
                parents: 0,
                interior: {
                  X1: { PalletInstance: balancesPalletIndex },
                },
              },
            },
            fun: { Fungible: 1 },
          },
        ],
      },
      {
        BuyExecution: {
          fees: {
            id: {
              Concrete: {
                parents: 0,
                interior: {
                  X1: { PalletInstance: balancesPalletIndex },
                },
              },
            },
            fun: { Fungible: 1 },
          },
          weightLimit: { Limited: new BN(20000000000) },
        },
      },
    ];

    const xcmMessageNotExecuted = {
      V2: instructionsNotExecuted,
    };

    const xcmpFormat: XcmpMessageFormat = context.polkadotApi.createType(
      "XcmpMessageFormat",
      "ConcatenatedVersionedXcm"
    ) as any;
    const receivedMessageNotExecuted: XcmVersionedXcm = context.polkadotApi.createType(
      "XcmVersionedXcm",
      xcmMessageNotExecuted
    ) as any;

    const totalMessageNotExecuted = [...xcmpFormat.toU8a(), ...receivedMessageNotExecuted.toU8a()];

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
      await customWeb3Request(context.web3, "xcm_injectHrmpMessage", [1, totalMessageNotExecuted]);
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
    // We first simulate a reception for suspending a channel from parachain 1
    const xcmpFormat: XcmpMessageFormat = context.polkadotApi.createType(
      "XcmpMessageFormat",
      "Signals"
    ) as any;
    const receivedMessage = context.polkadotApi.createType("u8", 0) as any;
    const totalMessage = [...xcmpFormat.toU8a(), ...receivedMessage.toU8a()];
    // Send RPC call to inject XCM message
    // We will set a specific message knowing that it should mint the statemint asset
    await customWeb3Request(context.web3, "xcm_injectHrmpMessage", [suspendedPara, totalMessage]);

    // Create a block in which the XCM will be executed
    await context.createBlock();

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

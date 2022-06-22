import "@moonbeam-network/api-augment";

import { BN, u8aToHex } from "@polkadot/util";
import { expect } from "chai";

import { alith } from "../../util/accounts";
import { RELAY_SOURCE_LOCATION } from "../../util/assets";
import { customWeb3Request } from "../../util/providers";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";

import type { XcmVersionedXcm } from "@polkadot/types/lookup";

// Twelve decimal places in the moonbase relay chain's token
const RELAY_TOKEN = 1_000_000_000_000n;

const palletId = "0x6D6f646c617373746d6E67720000000000000000";

const assetMetadata = {
  name: "DOT",
  symbol: "DOT",
  decimals: new BN(12),
  isFrozen: false,
};

describeDevMoonbeam("Mock XCM - receive downward transfer", (context) => {
  let assetId: string;

  before("Should Register an asset and set unit per sec", async function () {
    // registerForeignAsset
    const {
      result: { events: eventsRegister },
    } = await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.assetManager.registerForeignAsset(
          RELAY_SOURCE_LOCATION,
          assetMetadata,
          new BN(1),
          true
        )
      )
    );
    // Look for assetId in events
    assetId = eventsRegister
      .find(({ event: { section } }) => section.toString() === "assetManager")
      .event.data[0].toHex()
      .replace(/,/g, "");

    // setAssetUnitsPerSecond
    const {
      result: { events },
    } = await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.assetManager.setAssetUnitsPerSecond(RELAY_SOURCE_LOCATION, 0, 0)
      )
    );
    expect(events[1].event.method.toString()).to.eq("UnitsPerSecondChanged");
    expect(events[4].event.method.toString()).to.eq("ExtrinsicSuccess");

    // check asset in storage
    const registeredAsset = (
      (await context.polkadotApi.query.assets.asset(assetId)) as any
    ).unwrap();
    expect(registeredAsset.owner.toHex()).to.eq(palletId.toLowerCase());
  });

  it("Should receive a downward transfer of 10 DOTs to Alith", async function () {
    // Send RPC call to inject XCM message
    // You can provide a message, but if you don't a downward transfer is the default
    await customWeb3Request(context.web3, "xcm_injectDownwardMessage", [[]]);

    // Create a block in which the XCM will be executed
    await context.createBlock();

    // Make sure the state has ALITH's to DOT tokens
    let alith_dot_balance = (
      (await context.polkadotApi.query.assets.account(assetId, alith.address)) as any
    )
      .unwrap()
      ["balance"].toBigInt();

    expect(alith_dot_balance).to.eq(10n * RELAY_TOKEN);
  });
});

describeDevMoonbeam("Mock XCMP - test XCMP execution", (context) => {
  it("Should test DMP on_initialization and on_idle", async function () {
    const metadata = await context.polkadotApi.rpc.state.getMetadata();
    const balancesPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find(
      (pallet) => pallet.name === "Balances"
    ).index;
    let ownParaId = (await context.polkadotApi.query.parachainInfo.parachainId()) as any;

    let numMsgs = 50;
    // let's target half of then being executed

    // xcmp reserved is BLOCK/4
    const totalDmpWeight =
      context.polkadotApi.consts.system.blockWeights.maxBlock.toBigInt() / BigInt(4);

    // we want half of numParaMsgs to be executed. That give us how much each message weights
    const weightPerMessage = (totalDmpWeight * BigInt(2)) / BigInt(numMsgs);

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

    const receivedMessage: XcmVersionedXcm = context.polkadotApi.createType(
      "XcmVersionedXcm",
      xcmMessage
    ) as any;

    const totalMessage = [...receivedMessage.toU8a()];

    // We want these isntructions to fail in BuyExecution. That means
    // WithdrawAsset needs to work. The only way for this to work
    // is to fund each sovereign account
    const sovereignAddress = u8aToHex(
      new Uint8Array([...new TextEncoder().encode("Parent")])
    ).padEnd(42, "0");

    // We first fund the parent sovereign account with 1000
    // we will only withdraw 1, so no problem on this
    await context.createBlock(context.polkadotApi.tx.balances.transfer(sovereignAddress, 1000n));

    // now we start injecting messages
    // several
    for (let i = 0; i < numMsgs; i++) {
      await customWeb3Request(context.web3, "xcm_injectDownwardMessage", [totalMessage]);
    }

    await context.createBlock();

    const signedBlock = await context.polkadotApi.rpc.chain.getBlock();
    const apiAt = await context.polkadotApi.at(signedBlock.block.header.hash);
    const allRecords = await apiAt.query.system.events();
    const events = allRecords.map(({ event }) => `${event.section}.${event.method}.${event.data}`);

    // lets grab at which point the dmp queue was exhausted
    let index = events.findIndex((event) => {
      if (event.includes("dmpQueue.WeightExhausted.")) {
        return true;
      } else {
        return false;
      }
    });
    const eventsExecutedOnInitialization = events.slice(0, index + 1);
    const eventsExecutedOnIdle = events.slice(index + 1, events.length);

    // lets count
    let executedOnIdle = 0;
    let executedOnInitialization = 0;

    // OnInitialization
    eventsExecutedOnInitialization.forEach((e) => {
      if (e.toString().includes("tooExpensive")) {
        executedOnInitialization += 1;
      }
    });

    // OnIdle
    eventsExecutedOnIdle.forEach((e) => {
      if (e.toString().includes("tooExpensive")) {
        executedOnIdle += 1;
      }
    });

    // the test was designed to go half and half
    expect(executedOnInitialization).to.be.eq(25);
    expect(executedOnIdle).to.be.eq(25);
    const pageIndex = await apiAt.query.dmpQueue.pageIndex();
    expect(pageIndex.beginUsed.toBigInt()).to.eq(0n);
    expect(pageIndex.endUsed.toBigInt()).to.eq(0n);
  });
});

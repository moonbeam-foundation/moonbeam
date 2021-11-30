import { expect } from "chai";

import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createTransfer } from "../../util/transactions";

describeDevMoonbeam("Balance extrinsics", (context) => {
  it("should appear after transfer", async function () {
    const testAddress = "0x1111111111111111111111111111111111111111";
    await context.createBlock({
      transactions: [await createTransfer(context, testAddress, 512)],
    });

    const blockHash = await context.polkadotApi.rpc.chain.getBlockHash(1);
    const signedBlock = await context.polkadotApi.rpc.chain.getBlock(blockHash);
    const allRecords = await context.polkadotApi.query.system.events.at(
      signedBlock.block.header.hash
    );

    // map between the extrinsics and events
    signedBlock.block.extrinsics.forEach(({ method: { method, section } }, index) => {
      // filter the specific events based on the phase and then the
      // index of our extrinsic in the block
      const events = allRecords
        .filter(({ phase }) => phase.isApplyExtrinsic && phase.asApplyExtrinsic.eq(index))
        .map(({ event }) => event);

      switch (index) {
        // First 3 events:
        // timestamp.set:: system.ExtrinsicSuccess
        // parachainUpgrade.setValidationData:: system.ExtrinsicSuccess
        // authorInherent.setAuthor:: system.ExtrinsicSuccess
        case 0:
        case 1:
        case 2:
          expect(
            events.length === 1 && context.polkadotApi.events.system.ExtrinsicSuccess.is(events[0])
          ).to.be.true;
          break;
        // Fourth event: ethereum.transact:: system.NewAccount, balances.Endowed, (?),
        // ethereum.Executed, system.ExtrinsicSuccess
        case 3:
          expect(section === "ethereum" && method === "transact").to.be.true;
          expect(events.length === 4);
          expect(context.polkadotApi.events.system.NewAccount.is(events[1])).to.be.true;
          expect(context.polkadotApi.events.balances.Endowed.is(events[2])).to.be.true;
          // TODO: what event was inserted here?
          expect(context.polkadotApi.events.balances.Endowed.is(events[7])).to.be.true; // treasury
          expect(context.polkadotApi.events.treasury.Deposit.is(events[8])).to.be.true;
          expect(context.polkadotApi.events.ethereum.Executed.is(events[9])).to.be.true;
          expect(context.polkadotApi.events.system.ExtrinsicSuccess.is(events[10])).to.be.true;
          break;
        default:
          throw new Error(`Unexpected extrinsic`);
      }
    });
  });
});

describeDevMoonbeam("Balance extrinsics (EIP2930)", (context) => {
  it("should appear after transfer", async function () {
    const testAddress = "0x1111111111111111111111111111111111111111";
    await context.createBlock({
      transactions: [await createTransfer(context, testAddress, 512, { accessList: [] })],
    });

    const blockHash = await context.polkadotApi.rpc.chain.getBlockHash(1);
    const signedBlock = await context.polkadotApi.rpc.chain.getBlock(blockHash);
    const allRecords = await context.polkadotApi.query.system.events.at(
      signedBlock.block.header.hash
    );

    // map between the extrinsics and events
    signedBlock.block.extrinsics.forEach(({ method: { method, section } }, index) => {
      // filter the specific events based on the phase and then the
      // index of our extrinsic in the block
      const events = allRecords
        .filter(({ phase }) => phase.isApplyExtrinsic && phase.asApplyExtrinsic.eq(index))
        .map(({ event }) => event);

      switch (index) {
        // First 3 events:
        // timestamp.set:: system.ExtrinsicSuccess
        // parachainUpgrade.setValidationData:: system.ExtrinsicSuccess
        // authorInherent.setAuthor:: system.ExtrinsicSuccess
        case 0:
        case 1:
        case 2:
          expect(
            events.length === 1 && context.polkadotApi.events.system.ExtrinsicSuccess.is(events[0])
          ).to.be.true;
          break;
        // Fourth event: ethereum.transact:: system.NewAccount, balances.Endowed, (?),
        // ethereum.Executed, system.ExtrinsicSuccess
        case 3:
          expect(section === "ethereum" && method === "transact").to.be.true;
          expect(events.length === 4);
          expect(context.polkadotApi.events.system.NewAccount.is(events[1])).to.be.true;
          expect(context.polkadotApi.events.balances.Endowed.is(events[2])).to.be.true;
          // TODO: what event was inserted here?
          expect(context.polkadotApi.events.balances.Endowed.is(events[7])).to.be.true; // treasury
          expect(context.polkadotApi.events.treasury.Deposit.is(events[8])).to.be.true;
          expect(context.polkadotApi.events.ethereum.Executed.is(events[9])).to.be.true;
          expect(context.polkadotApi.events.system.ExtrinsicSuccess.is(events[10])).to.be.true;
          break;
        default:
          throw new Error(`Unexpected extrinsic`);
      }
    });
  });
});

describeDevMoonbeam("Balance extrinsics (EIP1559)", (context) => {
  it("should appear after transfer", async function () {
    const testAddress = "0x1111111111111111111111111111111111111111";
    await context.createBlock({
      transactions: [await createTransfer(context, testAddress, 512, { maxFeePerGas: 1_000_000_000 })],
    });

    const blockHash = await context.polkadotApi.rpc.chain.getBlockHash(1);
    const signedBlock = await context.polkadotApi.rpc.chain.getBlock(blockHash);
    const allRecords = await context.polkadotApi.query.system.events.at(
      signedBlock.block.header.hash
    );

    // map between the extrinsics and events
    signedBlock.block.extrinsics.forEach(({ method: { method, section } }, index) => {
      // filter the specific events based on the phase and then the
      // index of our extrinsic in the block
      const events = allRecords
        .filter(({ phase }) => phase.isApplyExtrinsic && phase.asApplyExtrinsic.eq(index))
        .map(({ event }) => event);

      switch (index) {
        // First 3 events:
        // timestamp.set:: system.ExtrinsicSuccess
        // parachainUpgrade.setValidationData:: system.ExtrinsicSuccess
        // authorInherent.setAuthor:: system.ExtrinsicSuccess
        case 0:
        case 1:
        case 2:
          expect(
            events.length === 1 && context.polkadotApi.events.system.ExtrinsicSuccess.is(events[0])
          ).to.be.true;
          break;
        // Fourth event: ethereum.transact:: system.NewAccount, balances.Endowed, (?),
        // ethereum.Executed, system.ExtrinsicSuccess
        case 3:
          expect(section === "ethereum" && method === "transact").to.be.true;
          expect(events.length === 4);
          expect(context.polkadotApi.events.system.NewAccount.is(events[1])).to.be.true;
          expect(context.polkadotApi.events.balances.Endowed.is(events[2])).to.be.true;
          // TODO: what event was inserted here?
          expect(context.polkadotApi.events.balances.Endowed.is(events[7])).to.be.true; // treasury
          expect(context.polkadotApi.events.treasury.Deposit.is(events[8])).to.be.true;
          expect(context.polkadotApi.events.ethereum.Executed.is(events[9])).to.be.true;
          expect(context.polkadotApi.events.system.ExtrinsicSuccess.is(events[10])).to.be.true;
          break;
        default:
          throw new Error(`Unexpected extrinsic`);
      }
    });
  });
});

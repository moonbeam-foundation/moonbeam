import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { alith, generateKeyringPair } from "../../../util/accounts";
import { mapExtrinsics } from "../../../util/block";
import { describeDevMoonbeamAllEthTxTypes } from "../../../util/setup-dev-tests";
import { createTransfer } from "../../../util/transactions";

describeDevMoonbeamAllEthTxTypes("Balance - Extrinsic", (context) => {
  const randomAccount = generateKeyringPair();
  it("should emit ethereum/transfer events", async function () {
    await context.createBlock(createTransfer(context, randomAccount.address, 512));

    const blockHash = await context.polkadotApi.rpc.chain.getBlockHash(1);
    const signedBlock = await context.polkadotApi.rpc.chain.getBlock(blockHash);
    const allRecords = (await context.polkadotApi.query.system.events.at(
      signedBlock.block.header.hash
    )) as any;

    const txsWithEvents = mapExtrinsics(signedBlock.block.extrinsics, allRecords);

    const ethTx = txsWithEvents.find(({ extrinsic: { method } }) => method.section == "ethereum");

    expect(ethTx.events.length).to.eq(11);
    expect(context.polkadotApi.events.system.NewAccount.is(ethTx.events[1])).to.be.true;
    expect(context.polkadotApi.events.balances.Endowed.is(ethTx.events[2])).to.be.true;
    expect(context.polkadotApi.events.balances.Transfer.is(ethTx.events[3])).to.be.true;
    expect(ethTx.events[3].data[0].toString()).to.eq(alith.address);
    expect(ethTx.events[3].data[1].toString()).to.eq(randomAccount.address);
    expect(context.polkadotApi.events.balances.Endowed.is(ethTx.events[7])).to.be.true; // treasury
    expect(context.polkadotApi.events.treasury.Deposit.is(ethTx.events[8])).to.be.true;
    expect(context.polkadotApi.events.ethereum.Executed.is(ethTx.events[9])).to.be.true;
    expect(context.polkadotApi.events.system.ExtrinsicSuccess.is(ethTx.events[10])).to.be.true;
  });
});

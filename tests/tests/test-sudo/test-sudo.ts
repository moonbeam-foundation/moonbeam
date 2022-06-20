import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { alith, ALITH_GENESIS_BALANCE, baltathar } from "../../util/accounts";
import { verifyLatestBlockFees } from "../../util/block";
import { DEFAULT_GENESIS_BALANCE, ZERO_ADDRESS } from "../../util/constants";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createTransfer } from "../../util/transactions";

describeDevMoonbeam("Sudo - successful setParachainBondAccount", (context) => {
  it("should be able to call sudo with the right account", async function () {
    const {
      result: { events },
    } = await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.parachainStaking.setParachainBondAccount(alith.address)
      )
    );
    //check parachainBondInfo
    const parachainBondInfo = await context.polkadotApi.query.parachainStaking.parachainBondInfo();
    expect(parachainBondInfo.account.toString()).to.equal(alith.address);
    expect(parachainBondInfo.percent.toNumber()).to.equal(30);
    //check events
    expect(events.length).to.eq(5);
    expect(context.polkadotApi.events.parachainStaking.ParachainBondAccountSet.is(events[1].event))
      .to.be.true;
    expect(context.polkadotApi.events.balances.Deposit.is(events[3].event)).to.be.true;
    expect(context.polkadotApi.events.system.ExtrinsicSuccess.is(events[4].event)).to.be.true;
    // check balance diff (diff should be null for sudo - funds are sent back)
    expect(await context.web3.eth.getBalance(alith.address, 1)).to.equal(
      ALITH_GENESIS_BALANCE.toString()
    );
  });
});
describeDevMoonbeam("Sudo - fail if no funds in sudo", (context) => {
  before("Setup genesis account for substrate", async () => {
    const initBalance = await context.web3.eth.getBalance(alith.address);
    await context.createBlock(
      createTransfer(
        context,
        baltathar.address,
        BigInt(initBalance) - 1n - 21000n * 1_000_000_000n,
        {
          gas: 21000,
        }
      )
    );
    expect(await context.web3.eth.getBalance(alith.address)).to.eq("1");
  });
  it("should not be able to call sudo with no funds", async function () {
    try {
      await context.createBlock(
        context.polkadotApi.tx.sudo.sudo(
          context.polkadotApi.tx.parachainStaking.setParachainBondAccount(alith.address)
        )
      );
    } catch (e) {
      expect(e.toString()).to.eq(
        "RpcError: 1010: Invalid Transaction: Inability " +
          "to pay some fees , e.g. account balance too low"
      );
    }
    //check parachainBondInfo
    const parachainBondInfo = await context.polkadotApi.query.parachainStaking.parachainBondInfo();
    expect(parachainBondInfo.account.toString()).to.equal(ZERO_ADDRESS);
  });
});
describeDevMoonbeam("Sudo - Only sudo account", (context) => {
  it("should NOT be able to call sudo with another account than sudo account", async function () {
    const {
      result: { events },
    } = await context.createBlock(
      context.polkadotApi.tx.sudo
        .sudo(context.polkadotApi.tx.parachainStaking.setParachainBondAccount(alith.address))
        .signAsync(baltathar)
    );
    //check parachainBondInfo
    const parachainBondInfo = await context.polkadotApi.query.parachainStaking.parachainBondInfo();
    expect(parachainBondInfo.account.toString()).to.equal(ZERO_ADDRESS);
    expect(parachainBondInfo.percent.toNumber()).to.equal(30);
    //check events
    expect(events.length === 6).to.be.true;
    expect(context.polkadotApi.events.system.NewAccount.is(events[2].event)).to.be.true;
    expect(context.polkadotApi.events.balances.Endowed.is(events[3].event)).to.be.true;
    expect(context.polkadotApi.events.treasury.Deposit.is(events[4].event)).to.be.true;
    expect(context.polkadotApi.events.system.ExtrinsicFailed.is(events[5].event)).to.be.true;
    // check balance diff (should not be null for a failed extrinsic)
    expect(
      BigInt(await context.web3.eth.getBalance(baltathar.address, 1)) - DEFAULT_GENESIS_BALANCE !==
        0n
    ).to.equal(true);
  });
});

describeDevMoonbeam("Sudo - Only sudo account - test gas", (context) => {
  it("should NOT be able to call sudo with another account than sudo account", async function () {
    await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.parachainStaking.setParachainBondAccount(alith.address)
      )
    );

    await verifyLatestBlockFees(context);
  });
});

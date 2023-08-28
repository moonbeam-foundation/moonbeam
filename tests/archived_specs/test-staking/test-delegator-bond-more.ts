import "@polkadot/api-augment";
import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { MIN_GLMR_STAKING } from "../../util/constants";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { alith, ethan } from "../../util/accounts";
import { expectOk } from "../../util/expect";

describeDevMoonbeam("Staking - Bond More - no scheduled request", (context) => {
  before("should delegate", async () => {
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .delegate(alith.address, MIN_GLMR_STAKING * 5n, 0, 0)
          .signAsync(ethan)
      )
    );
  });

  it("should succeed and increase total", async () => {
    const bondAmountBefore = (
      await context.polkadotApi.query.parachainStaking.delegatorState(ethan.address)
    ).unwrap().total;

    const increaseAmount = 5;
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegatorBondMore(alith.address, increaseAmount)
        .signAsync(ethan)
    );

    const bondAmountAfter = (
      await context.polkadotApi.query.parachainStaking.delegatorState(ethan.address)
    ).unwrap().total;
    expect(bondAmountAfter.eq(bondAmountBefore.addn(increaseAmount))).to.be.true;
  });
});

describeDevMoonbeam("Staking - Bond More - bond less scheduled", (context) => {
  before("should schedule bond less", async () => {
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .delegate(alith.address, MIN_GLMR_STAKING * 5n, 0, 0)
          .signAsync(ethan)
      )
    );

    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .scheduleDelegatorBondLess(alith.address, 10n)
        .signAsync(ethan)
    );
  });

  it("should succeed and increase total", async () => {
    const bondAmountBefore = (
      await context.polkadotApi.query.parachainStaking.delegatorState(ethan.address)
    ).unwrap().total;

    const increaseAmount = 5;
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegatorBondMore(alith.address, increaseAmount)
        .signAsync(ethan)
    );

    const bondAmountAfter = (
      await context.polkadotApi.query.parachainStaking.delegatorState(ethan.address)
    ).unwrap().total;
    expect(bondAmountAfter.eq(bondAmountBefore.addn(increaseAmount))).to.be.true;
  });
});

describeDevMoonbeam("Staking - Bond More - revoke scheduled", (context) => {
  before("should delegate", async () => {
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .delegate(alith.address, MIN_GLMR_STAKING * 5n, 0, 0)
          .signAsync(ethan)
      )
    );

    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .scheduleRevokeDelegation(alith.address)
        .signAsync(ethan)
    );
  });

  it("should fail", async () => {
    const bondAmountBefore = (
      await context.polkadotApi.query.parachainStaking.delegatorState(ethan.address)
    ).unwrap().total;

    const increaseAmount = 5n;
    const block = await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegatorBondMore(alith.address, increaseAmount)
        .signAsync(ethan)
    );

    expect(block.result.error.name).to.equal("PendingDelegationRevoke");
    const bondAmountAfter = (
      await context.polkadotApi.query.parachainStaking.delegatorState(ethan.address)
    ).unwrap().total;
    expect(bondAmountAfter.eq(bondAmountBefore)).to.be.true;
  });
});

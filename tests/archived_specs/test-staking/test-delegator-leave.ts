import "@polkadot/api-augment";
import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { MIN_GLMR_STAKING, MIN_GLMR_DELEGATOR } from "../../util/constants";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { alith, baltathar, ethan } from "../../util/accounts";
import { expectOk } from "../../util/expect";
import { jumpRounds } from "../../util/block";

describeDevMoonbeam("Staking - Delegator Leave Schedule - already scheduled", (context) => {
  before("should delegate", async () => {
    await expectOk(
      context.createBlock([
        context.polkadotApi.tx.parachainStaking
          .delegate(alith.address, MIN_GLMR_DELEGATOR, 0, 0)
          .signAsync(ethan),
      ])
    );

    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking.scheduleLeaveDelegators().signAsync(ethan)
      )
    );
  });

  it("should fail", async () => {
    const block = await context.createBlock(
      context.polkadotApi.tx.parachainStaking.scheduleLeaveDelegators().signAsync(ethan)
    );
    expect(block.result.successful).to.be.false;
    expect(block.result.error.name).to.equal("DelegatorAlreadyLeaving");
  });
});

describeDevMoonbeam("Staking - Delegator Leave Schedule - valid request", (context) => {
  before("should delegate", async () => {
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .delegate(alith.address, MIN_GLMR_DELEGATOR, 0, 0)
          .signAsync(ethan)
      )
    );

    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking.scheduleLeaveDelegators().signAsync(ethan)
      )
    );
  });

  it("should schedule revokes on all delegations", async () => {
    const delegatorState = (
      await context.polkadotApi.query.parachainStaking.delegatorState(ethan.address)
    ).unwrap();
    const currentRound = (
      await context.polkadotApi.query.parachainStaking.round()
    ).current.toNumber();
    const roundDelay = context.polkadotApi.consts.parachainStaking.leaveDelegatorsDelay.toNumber();

    for await (const delegation of delegatorState.delegations) {
      const scheduledRequests =
        (await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(
          delegation.owner
        )) as unknown as any[];
      const revokeRequest = scheduledRequests.find(
        (req) => req.delegator.eq(ethan.address) && req.action.isRevoke
      );
      expect(revokeRequest).to.not.be.undefined;
      expect(revokeRequest.whenExecutable.toNumber()).to.equal(currentRound + roundDelay);
    }
  });
});

describeDevMoonbeam("Staking - Delegator Leave Execute - before round delay", (context) => {
  before("should delegate, schedule leave, and jump to earlier round", async () => {
    await expectOk(
      context.createBlock([
        context.polkadotApi.tx.sudo
          .sudo(context.polkadotApi.tx.parachainStaking.setBlocksPerRound(10))
          .signAsync(alith),
        context.polkadotApi.tx.parachainStaking
          .delegate(alith.address, MIN_GLMR_DELEGATOR, 0, 0)
          .signAsync(ethan),
      ])
    );
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking.scheduleLeaveDelegators().signAsync(ethan)
      )
    );

    const leaveDelay = context.polkadotApi.consts.parachainStaking.leaveDelegatorsDelay;
    await jumpRounds(context, leaveDelay.subn(1).toNumber());
  });

  it("should fail", async () => {
    const block = await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .executeLeaveDelegators(ethan.address, 1)
        .signAsync(ethan)
    );
    expect(block.result.successful).to.be.false;
    expect(block.result.error.name).to.equal("DelegatorCannotLeaveYet");
  });
});

describeDevMoonbeam("Staking - Delegator Leave - exact round delay", (context) => {
  before("should delegate, schedule leave, and jump to exact round", async () => {
    await expectOk(
      context.createBlock([
        context.polkadotApi.tx.sudo
          .sudo(context.polkadotApi.tx.parachainStaking.setBlocksPerRound(10))
          .signAsync(alith),
        context.polkadotApi.tx.parachainStaking
          .delegate(alith.address, MIN_GLMR_DELEGATOR, 0, 0)
          .signAsync(ethan),
      ])
    );
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking.scheduleLeaveDelegators().signAsync(ethan)
      )
    );

    const leaveDelay = context.polkadotApi.consts.parachainStaking.leaveDelegatorsDelay;
    await jumpRounds(context, leaveDelay.toNumber());
  });

  it("should succeed", async () => {
    const block = await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .executeLeaveDelegators(ethan.address, 1)
        .signAsync(ethan)
    );
    expect(block.result.successful).to.be.true;
    const leaveEvents = block.result.events.reduce((acc, event) => {
      if (context.polkadotApi.events.parachainStaking.DelegatorLeft.is(event.event)) {
        acc.push({
          account: event.event.data[0].toString(),
        });
      }
      return acc;
    }, []);

    expect(leaveEvents).to.deep.equal([
      {
        account: ethan.address,
      },
    ]);
  });
});

describeDevMoonbeam(
  "Staking - Delegator Leave - executeLeaveDelegators executed after round delay",
  (context) => {
    const EXTRA_BOND_AMOUNT = 1_000_000_000_000_000_000n;
    const BOND_AMOUNT = MIN_GLMR_STAKING + EXTRA_BOND_AMOUNT;

    before("should delegate, schedule leave, and jump to after round", async () => {
      await expectOk(
        context.createBlock([
          context.polkadotApi.tx.sudo
            .sudo(context.polkadotApi.tx.parachainStaking.setBlocksPerRound(10))
            .signAsync(alith),
          context.polkadotApi.tx.parachainStaking
            .delegate(alith.address, MIN_GLMR_DELEGATOR, 0, 0)
            .signAsync(ethan),
        ])
      );
      await expectOk(
        context.createBlock(
          context.polkadotApi.tx.parachainStaking.scheduleLeaveDelegators().signAsync(ethan)
        )
      );

      const leaveDelay = context.polkadotApi.consts.parachainStaking.leaveDelegatorsDelay;
      await jumpRounds(context, leaveDelay.addn(5).toNumber());
    });

    it("should succeed", async () => {
      const block = await context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .executeLeaveDelegators(ethan.address, 1)
          .signAsync(ethan)
      );
      expect(block.result.successful).to.be.true;
      const leaveEvents = block.result.events.reduce((acc, event) => {
        if (context.polkadotApi.events.parachainStaking.DelegatorLeft.is(event.event)) {
          acc.push({
            account: event.event.data[0].toString(),
          });
        }
        return acc;
      }, []);

      expect(leaveEvents).to.deep.equal([
        {
          account: ethan.address,
        },
      ]);
    });
  }
);

describeDevMoonbeam(
  "Staking - Delegator Leave Cancel - one revoke manually cancelled",
  (context) => {
    before("should schedule leave then cancel single revoke", async () => {
      await expectOk(
        context.createBlock([
          context.polkadotApi.tx.sudo
            .sudo(context.polkadotApi.tx.parachainStaking.setBlocksPerRound(10))
            .signAsync(alith),
          context.polkadotApi.tx.parachainStaking
            .joinCandidates(MIN_GLMR_STAKING, 1)
            .signAsync(baltathar),
          context.polkadotApi.tx.parachainStaking
            .delegate(alith.address, MIN_GLMR_DELEGATOR, 0, 0)
            .signAsync(ethan),
        ])
      );
      await expectOk(
        context.createBlock(
          context.polkadotApi.tx.parachainStaking
            .delegate(baltathar.address, MIN_GLMR_DELEGATOR, 0, 1)
            .signAsync(ethan)
        )
      );

      await expectOk(
        context.createBlock(
          context.polkadotApi.tx.parachainStaking.scheduleLeaveDelegators().signAsync(ethan)
        )
      );
      const leaveDelay = context.polkadotApi.consts.parachainStaking.leaveDelegatorsDelay;
      await jumpRounds(context, leaveDelay.addn(1).toNumber());

      // cancel single request
      await expectOk(
        context.createBlock([
          context.polkadotApi.tx.parachainStaking
            .cancelDelegationRequest(baltathar.address)
            .signAsync(ethan),
        ])
      );
    });

    it("should fail", async () => {
      const block = await context.createBlock(
        context.polkadotApi.tx.parachainStaking.cancelLeaveDelegators().signAsync(ethan)
      );
      expect(block.result.error.name).to.equal("DelegatorNotLeaving");
    });
  }
);

describeDevMoonbeam("Staking - Delegator Leave Cancel - manually reschedule revoke", (context) => {
  before("should schedule leave then cancel single revoke then reschedule it", async () => {
    await expectOk(
      context.createBlock([
        context.polkadotApi.tx.sudo
          .sudo(context.polkadotApi.tx.parachainStaking.setBlocksPerRound(10))
          .signAsync(alith),
        context.polkadotApi.tx.parachainStaking
          .joinCandidates(MIN_GLMR_STAKING, 1)
          .signAsync(baltathar),
        context.polkadotApi.tx.parachainStaking
          .delegate(alith.address, MIN_GLMR_DELEGATOR, 0, 0)
          .signAsync(ethan),
      ])
    );
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .delegate(baltathar.address, MIN_GLMR_DELEGATOR, 0, 1)
          .signAsync(ethan)
      )
    );

    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking.scheduleLeaveDelegators().signAsync(ethan)
      )
    );
    const leaveDelay = context.polkadotApi.consts.parachainStaking.leaveDelegatorsDelay;
    await jumpRounds(context, leaveDelay.addn(1).toNumber());

    // cancel single revoke request
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .cancelDelegationRequest(baltathar.address)
          .signAsync(ethan)
      )
    );

    // reschedule single revoke request
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .scheduleRevokeDelegation(baltathar.address)
          .signAsync(ethan)
      )
    );
  });

  it("should succeed", async () => {
    const block = await context.createBlock(
      context.polkadotApi.tx.parachainStaking.cancelLeaveDelegators().signAsync(ethan)
    );
    expect(block.result.successful).to.be.true;
    const leaveEvents = block.result.events.reduce((acc, event) => {
      if (context.polkadotApi.events.parachainStaking.DelegatorExitCancelled.is(event.event)) {
        acc.push({
          account: event.event.data[0].toString(),
        });
      }
      return acc;
    }, []);
    expect(leaveEvents).to.deep.equal([
      {
        account: ethan.address,
      },
    ]);
  });
});

describeDevMoonbeam("Staking - Delegator Leave Execute - revoke manually cancelled", (context) => {
  before("should schedule leave then cancel single revoke", async () => {
    await expectOk(
      context.createBlock([
        context.polkadotApi.tx.sudo
          .sudo(context.polkadotApi.tx.parachainStaking.setBlocksPerRound(10))
          .signAsync(alith),
        context.polkadotApi.tx.parachainStaking
          .joinCandidates(MIN_GLMR_STAKING, 1)
          .signAsync(baltathar),
        context.polkadotApi.tx.parachainStaking
          .delegate(alith.address, MIN_GLMR_DELEGATOR, 0, 0)
          .signAsync(ethan),
      ])
    );
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .delegate(baltathar.address, MIN_GLMR_DELEGATOR, 0, 1)
          .signAsync(ethan)
      )
    );

    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking.scheduleLeaveDelegators().signAsync(ethan)
      )
    );
    const leaveDelay = context.polkadotApi.consts.parachainStaking.leaveDelegatorsDelay;
    await jumpRounds(context, leaveDelay.addn(1).toNumber());

    // cancel single revoke request
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .cancelDelegationRequest(baltathar.address)
          .signAsync(ethan)
      )
    );
  });

  it("should fail", async () => {
    const block = await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .executeLeaveDelegators(ethan.address, 2)
        .signAsync(ethan)
    );
    expect(block.result.error.name).to.equal("DelegatorNotLeaving");
  });
});

describeDevMoonbeam(
  "Staking - Delegator Leave Execute - manually rescheduled revoke",
  (context) => {
    before("should schedule leave then cancel single revoke then reschedule it", async () => {
      await expectOk(
        context.createBlock([
          context.polkadotApi.tx.sudo
            .sudo(context.polkadotApi.tx.parachainStaking.setBlocksPerRound(10))
            .signAsync(alith),
          context.polkadotApi.tx.parachainStaking
            .joinCandidates(MIN_GLMR_STAKING, 1)
            .signAsync(baltathar),
          context.polkadotApi.tx.parachainStaking
            .delegate(alith.address, MIN_GLMR_DELEGATOR, 0, 0)
            .signAsync(ethan),
        ])
      );
      await expectOk(
        context.createBlock(
          context.polkadotApi.tx.parachainStaking
            .delegate(baltathar.address, MIN_GLMR_DELEGATOR, 0, 1)
            .signAsync(ethan)
        )
      );

      await expectOk(
        context.createBlock(
          context.polkadotApi.tx.parachainStaking.scheduleLeaveDelegators().signAsync(ethan)
        )
      );
      const leaveDelay = context.polkadotApi.consts.parachainStaking.leaveDelegatorsDelay;
      await jumpRounds(context, leaveDelay.addn(1).toNumber());

      // cancel single revoke request
      await expectOk(
        context.createBlock(
          context.polkadotApi.tx.parachainStaking
            .cancelDelegationRequest(baltathar.address)
            .signAsync(ethan)
        )
      );

      // reschedule single revoke request
      await expectOk(
        context.createBlock(
          context.polkadotApi.tx.parachainStaking
            .scheduleRevokeDelegation(baltathar.address)
            .signAsync(ethan)
        )
      );
      await jumpRounds(context, leaveDelay.addn(1).toNumber());
    });

    it("should succeed", async () => {
      const block = await context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .executeLeaveDelegators(ethan.address, 2)
          .signAsync(ethan)
      );
      expect(block.result.successful).to.be.true;
      const leaveEvents = block.result.events.reduce((acc, event) => {
        if (context.polkadotApi.events.parachainStaking.DelegatorLeft.is(event.event)) {
          acc.push({
            account: event.event.data[0].toString(),
          });
        }
        return acc;
      }, []);
      expect(leaveEvents).to.deep.equal([
        {
          account: ethan.address,
        },
      ]);
    });
  }
);

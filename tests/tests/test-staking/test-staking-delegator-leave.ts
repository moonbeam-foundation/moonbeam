import "@polkadot/api-augment";
import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { MIN_GLMR_STAKING } from "../../util/constants";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { alith, baltathar, ethan } from "../../util/accounts";
import { expectOk } from "../../util/expect";
import { jumpToRound } from ".";

describeDevMoonbeam(
  "Staking - Delegator Leave - executeLeaveDelegators executed after round delay",
  (context) => {
    const EXTRA_BOND_AMOUNT = 1_000_000_000_000_000_000n;
    const BOND_AMOUNT = MIN_GLMR_STAKING + EXTRA_BOND_AMOUNT;

    before("should scheduleLeaveDelegators", async () => {
      await expectOk(
        context.createBlock([
          context.polkadotApi.tx.sudo
            .sudo(context.polkadotApi.tx.parachainStaking.setBlocksPerRound(10))
            .signAsync(alith),
          context.polkadotApi.tx.parachainStaking
            .delegate(alith.address, BOND_AMOUNT, 0, 0)
            .signAsync(ethan),
        ])
      );
      await expectOk(
        context.createBlock(
          context.polkadotApi.tx.parachainStaking.scheduleLeaveDelegators().signAsync(ethan)
        )
      );

      const leaveDelay = context.polkadotApi.consts.parachainStaking.leaveDelegatorsDelay;
      const currentRound = (await context.polkadotApi.query.parachainStaking.round()).current;
      await jumpToRound(context, currentRound.add(leaveDelay).addn(1).toNumber());
    });

    it("should allow executeLeaveDelegators to be executed", async function () {
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
  "Staking - Delegator Leave - cancelLeaveDelegators fails if revoke manually cancelled",
  (context) => {
    const EXTRA_BOND_AMOUNT = 1_000_000_000_000_000_000n;
    const BOND_AMOUNT = MIN_GLMR_STAKING + EXTRA_BOND_AMOUNT;

    before("should  schedule leave then cancel single revoke", async () => {
      await expectOk(
        context.createBlock([
          context.polkadotApi.tx.sudo
            .sudo(context.polkadotApi.tx.parachainStaking.setBlocksPerRound(10))
            .signAsync(alith),
          context.polkadotApi.tx.parachainStaking
            .joinCandidates(MIN_GLMR_STAKING, 1)
            .signAsync(baltathar),
          context.polkadotApi.tx.parachainStaking
            .delegate(alith.address, BOND_AMOUNT, 0, 0)
            .signAsync(ethan),
        ])
      );
      await expectOk(
        context.createBlock(
          context.polkadotApi.tx.parachainStaking
            .delegate(baltathar.address, BOND_AMOUNT, 0, 1)
            .signAsync(ethan)
        )
      );

      await expectOk(
        context.createBlock(
          context.polkadotApi.tx.parachainStaking.scheduleLeaveDelegators().signAsync(ethan)
        )
      );
      const leaveDelay = context.polkadotApi.consts.parachainStaking.leaveDelegatorsDelay;
      const currentRound = (await context.polkadotApi.query.parachainStaking.round()).current;
      await jumpToRound(context, currentRound.add(leaveDelay).addn(1).toNumber());

      // cancel single request
      await expectOk(
        context.createBlock([
          context.polkadotApi.tx.parachainStaking
            .cancelDelegationRequest(baltathar.address)
            .signAsync(ethan),
        ])
      );
    });

    it("should not allow cancelLeaveDelegators to be executed", async () => {
      const block = await context.createBlock(
        context.polkadotApi.tx.parachainStaking.cancelLeaveDelegators().signAsync(ethan)
      );
      expect(block.result.error.name).to.equal("DelegatorNotLeaving");
    });
  }
);

describeDevMoonbeam(
  "Staking - Delegator Leave - cancelLeaveDelegators executes with manually rescheduled revoke",
  (context) => {
    const EXTRA_BOND_AMOUNT = 1_000_000_000_000_000_000n;
    const BOND_AMOUNT = MIN_GLMR_STAKING + EXTRA_BOND_AMOUNT;

    before("should schedule leave then cancel single revoke then reschedule it after", async () => {
      await expectOk(
        context.createBlock([
          context.polkadotApi.tx.sudo
            .sudo(context.polkadotApi.tx.parachainStaking.setBlocksPerRound(10))
            .signAsync(alith),
          context.polkadotApi.tx.parachainStaking
            .joinCandidates(MIN_GLMR_STAKING, 1)
            .signAsync(baltathar),
          context.polkadotApi.tx.parachainStaking
            .delegate(alith.address, BOND_AMOUNT, 0, 0)
            .signAsync(ethan),
        ])
      );
      await expectOk(
        context.createBlock(
          context.polkadotApi.tx.parachainStaking
            .delegate(baltathar.address, BOND_AMOUNT, 0, 1)
            .signAsync(ethan)
        )
      );

      await expectOk(
        context.createBlock(
          context.polkadotApi.tx.parachainStaking.scheduleLeaveDelegators().signAsync(ethan)
        )
      );
      const leaveDelay = context.polkadotApi.consts.parachainStaking.leaveDelegatorsDelay;
      const currentRound = (await context.polkadotApi.query.parachainStaking.round()).current;
      await jumpToRound(context, currentRound.add(leaveDelay).addn(1).toNumber());

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

    it("should allow cancelLeaveDelegators to be executed", async function () {
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
  }
);

describeDevMoonbeam(
  "Staking - Delegator Leave - executeLeaveDelegators fails if revoke manually cancelled",
  (context) => {
    const EXTRA_BOND_AMOUNT = 1_000_000_000_000_000_000n;
    const BOND_AMOUNT = MIN_GLMR_STAKING + EXTRA_BOND_AMOUNT;

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
            .delegate(alith.address, BOND_AMOUNT, 0, 0)
            .signAsync(ethan),
        ])
      );
      await expectOk(
        context.createBlock(
          context.polkadotApi.tx.parachainStaking
            .delegate(baltathar.address, BOND_AMOUNT, 0, 1)
            .signAsync(ethan)
        )
      );

      await expectOk(
        context.createBlock(
          context.polkadotApi.tx.parachainStaking.scheduleLeaveDelegators().signAsync(ethan)
        )
      );
      const leaveDelay = context.polkadotApi.consts.parachainStaking.leaveDelegatorsDelay;
      const currentRound = (await context.polkadotApi.query.parachainStaking.round()).current;
      await jumpToRound(context, currentRound.add(leaveDelay).addn(1).toNumber());

      // cancel single revoke request
      await expectOk(
        context.createBlock(
          context.polkadotApi.tx.parachainStaking
            .cancelDelegationRequest(baltathar.address)
            .signAsync(ethan)
        )
      );
    });

    it("should not allow executeLeaveDelegators to be executed", async function () {
      const block = await context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .executeLeaveDelegators(ethan.address, 2)
          .signAsync(ethan)
      );
      expect(block.result.error.name).to.equal("DelegatorNotLeaving");
    });
  }
);

describeDevMoonbeam(
  "Staking - Delegator Leave - executeLeaveDelegators executes with manually rescheduled revoke",
  (context) => {
    const EXTRA_BOND_AMOUNT = 1_000_000_000_000_000_000n;
    const BOND_AMOUNT = MIN_GLMR_STAKING + EXTRA_BOND_AMOUNT;

    before("should schedule leave then cancel single revoke then reschedule it after", async () => {
      await expectOk(
        context.createBlock([
          context.polkadotApi.tx.sudo
            .sudo(context.polkadotApi.tx.parachainStaking.setBlocksPerRound(10))
            .signAsync(alith),
          context.polkadotApi.tx.parachainStaking
            .joinCandidates(MIN_GLMR_STAKING, 1)
            .signAsync(baltathar),
          context.polkadotApi.tx.parachainStaking
            .delegate(alith.address, BOND_AMOUNT, 0, 0)
            .signAsync(ethan),
        ])
      );
      await expectOk(
        context.createBlock(
          context.polkadotApi.tx.parachainStaking
            .delegate(baltathar.address, BOND_AMOUNT, 0, 1)
            .signAsync(ethan)
        )
      );

      await expectOk(
        context.createBlock(
          context.polkadotApi.tx.parachainStaking.scheduleLeaveDelegators().signAsync(ethan)
        )
      );
      const leaveDelay = context.polkadotApi.consts.parachainStaking.leaveDelegatorsDelay;
      const currentRound = (await context.polkadotApi.query.parachainStaking.round()).current;
      await jumpToRound(context, currentRound.add(leaveDelay).addn(1).toNumber());

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
      const newCurrentRound = (await context.polkadotApi.query.parachainStaking.round()).current;
      await jumpToRound(context, newCurrentRound.add(leaveDelay).addn(1).toNumber());
    });

    it("should allow executeLeaveDelegators to be executed", async function () {
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

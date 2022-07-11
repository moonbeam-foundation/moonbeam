import "@polkadot/api-augment";
import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { alith, ethan } from "../../util/accounts";
import { expectOk } from "../../util/expect";
import { jumpRounds } from "../../util/block";
import { MIN_GLMR_STAKING } from "../../util/constants";

describeDevMoonbeam("Staking - Candidate Leave Schedule - hint too low", (context) => {
  before("should join candidate", async () => {
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, 1).signAsync(ethan)
      )
    );
  });

  it("should fail", async () => {
    const block = await context.createBlock(
      context.polkadotApi.tx.parachainStaking.scheduleLeaveCandidates(1).signAsync(ethan)
    );
    expect(block.result.successful).to.be.false;
    expect(block.result.error.name).to.equal("TooLowCandidateCountToLeaveCandidates");
  });
});

describeDevMoonbeam("Staking - Candidate Leave Schedule - already scheduled", (context) => {
  before("should join candidate", async () => {
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, 1).signAsync(ethan)
      )
    );
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking.scheduleLeaveCandidates(2).signAsync(ethan)
      )
    );
  });

  it("should fail", async () => {
    const block = await context.createBlock(
      context.polkadotApi.tx.parachainStaking.scheduleLeaveCandidates(2).signAsync(ethan)
    );
    expect(block.result.successful).to.be.false;
    expect(block.result.error.name).to.equal("CandidateAlreadyLeaving");
  });
});

describeDevMoonbeam("Staking - Candidate Leave Schedule - valid request", (context) => {
  before("should join candidate", async () => {
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, 1).signAsync(ethan)
      )
    );
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking.scheduleLeaveCandidates(2).signAsync(ethan)
      )
    );
  });

  it("should change status to leaving at correct round", async () => {
    const candidatePool = (await context.polkadotApi.query.parachainStaking.candidatePool()).map(
      (c) => c.owner.toString()
    );
    const candidateState = (
      await context.polkadotApi.query.parachainStaking.candidateInfo(ethan.address)
    ).unwrap();
    const leaveDelay = context.polkadotApi.consts.parachainStaking.leaveCandidatesDelay;
    const currentRound = (await context.polkadotApi.query.parachainStaking.round()).current;

    expect(candidatePool).to.be.deep.equal([alith.address]);
    expect(candidateState.status.isLeaving).to.be.true;
    expect(candidateState.status.asLeaving.toNumber()).to.equal(
      currentRound.add(leaveDelay).toNumber()
    );
  });
});

describeDevMoonbeam("Staking - Candidate Leave Execute - before round delay", (context) => {
  before("should join candidates, schedule leave, and jump to earlier round", async () => {
    await expectOk(
      context.createBlock([
        context.polkadotApi.tx.sudo
          .sudo(context.polkadotApi.tx.parachainStaking.setBlocksPerRound(10))
          .signAsync(alith),
        context.polkadotApi.tx.parachainStaking
          .joinCandidates(MIN_GLMR_STAKING, 1)
          .signAsync(ethan),
      ])
    );
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking.scheduleLeaveCandidates(2).signAsync(ethan)
      )
    );

    const leaveDelay = context.polkadotApi.consts.parachainStaking.leaveDelegatorsDelay;
    await jumpRounds(context, leaveDelay.subn(1).toNumber());
  });

  it("should fail", async () => {
    const block = await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .executeLeaveCandidates(ethan.address, 0)
        .signAsync(ethan)
    );
    expect(block.result.successful).to.be.false;
    expect(block.result.error.name).to.equal("CandidateCannotLeaveYet");
  });
});

describeDevMoonbeam("Staking - Candidate Leave Execute - exact round delay", (context) => {
  before("should join candidates, schedule leave, and jump to exact round", async () => {
    await expectOk(
      context.createBlock([
        context.polkadotApi.tx.sudo
          .sudo(context.polkadotApi.tx.parachainStaking.setBlocksPerRound(10))
          .signAsync(alith),
        context.polkadotApi.tx.parachainStaking
          .joinCandidates(MIN_GLMR_STAKING, 1)
          .signAsync(ethan),
      ])
    );
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking.scheduleLeaveCandidates(2).signAsync(ethan)
      )
    );
    const leaveDelay = context.polkadotApi.consts.parachainStaking.leaveDelegatorsDelay;
    await jumpRounds(context, leaveDelay.toNumber());
  });

  it("should succeed", async () => {
    const block = await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .executeLeaveCandidates(ethan.address, 0)
        .signAsync(ethan)
    );
    expect(block.result.successful).to.be.true;
    const leaveEvents = block.result.events.reduce((acc, event) => {
      if (context.polkadotApi.events.parachainStaking.CandidateLeft.is(event.event)) {
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

describeDevMoonbeam("Staking - Candidate Leave Execute - after round delay", (context) => {
  before("should join candidates, schedule leave, and jump to after round", async () => {
    await expectOk(
      context.createBlock([
        context.polkadotApi.tx.sudo
          .sudo(context.polkadotApi.tx.parachainStaking.setBlocksPerRound(10))
          .signAsync(alith),
        context.polkadotApi.tx.parachainStaking
          .joinCandidates(MIN_GLMR_STAKING, 1)
          .signAsync(ethan),
      ])
    );

    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking.scheduleLeaveCandidates(2).signAsync(ethan)
      )
    );

    const leaveDelay = context.polkadotApi.consts.parachainStaking.leaveDelegatorsDelay;
    await jumpRounds(context, leaveDelay.addn(5).toNumber());
  });

  it("should succeed", async () => {
    const block = await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .executeLeaveCandidates(ethan.address, 0)
        .signAsync(ethan)
    );
    expect(block.result.successful).to.be.true;
    const leaveEvents = block.result.events.reduce((acc, event) => {
      if (context.polkadotApi.events.parachainStaking.CandidateLeft.is(event.event)) {
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

describeDevMoonbeam("Staking - Candidate Leave Cancel - leave not scheduled", (context) => {
  before("should join candidates", async () => {
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, 1).signAsync(ethan)
      )
    );
  });

  it("should fail", async () => {
    const block = await context.createBlock(
      context.polkadotApi.tx.parachainStaking.cancelLeaveCandidates(2).signAsync(ethan)
    );
    expect(block.result.error.name).to.equal("CandidateNotLeaving");
  });
});

describeDevMoonbeam("Staking - Candidate Leave Cancel - leave scheduled", (context) => {
  before("should join candidates and schedule leave", async () => {
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, 1).signAsync(ethan)
      )
    );
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking.scheduleLeaveCandidates(2).signAsync(ethan)
      )
    );
  });

  it("should succeed", async () => {
    const candidateStateBefore = (
      await context.polkadotApi.query.parachainStaking.candidateInfo(ethan.address)
    ).unwrap();
    expect(candidateStateBefore.status.isLeaving).to.be.true;

    const block = await context.createBlock(
      context.polkadotApi.tx.parachainStaking.cancelLeaveCandidates(2).signAsync(ethan)
    );
    expect(block.result.successful).to.be.true;

    const candidateStateAfter = (
      await context.polkadotApi.query.parachainStaking.candidateInfo(ethan.address)
    ).unwrap();
    expect(candidateStateAfter.status.isActive).to.be.true;
  });
});

import "@polkadot/api-augment";
import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { GLMR, MIN_GLMR_DELEGATOR, MIN_GLMR_STAKING } from "../../util/constants";
import { describeDevMoonbeam, DevTestContext } from "../../util/setup-dev-tests";
import { alith, baltathar, ethan, generateKeyringPair } from "../../util/accounts";
import { expectOk } from "../../util/expect";
import { jumpRounds } from "../../util/block";
import { BN, BN_ZERO } from "@polkadot/util";
import { Percent } from "../../util/common";
import { FrameSystemEventRecord } from "@polkadot/types/lookup";
import { KeyringPair } from "@polkadot/keyring/types";

describeDevMoonbeam("Staking - Rewards Auto-Compound - no auto-compound config", (context) => {
  before("should delegate", async () => {
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
  });

  it("should not compound reward and emit no event", async () => {
    const rewardDelay = context.polkadotApi.consts.parachainStaking.rewardPaymentDelay;
    await jumpRounds(context, rewardDelay.addn(1).toNumber());
    const blockHash = (await context.createBlock()).block.hash.toString();
    const events = await getRewardedAndCompoundedEvents(context, blockHash);
    const rewardedEvent = events.rewarded.find(({ account }) => account === ethan.address);
    const compoundedEvent = events.compounded.find(({ delegator }) => delegator === ethan.address);

    expect(rewardedEvent, "delegator was not rewarded").to.not.be.undefined;
    expect(compoundedEvent, "delegator reward was erroneously compounded").to.be.undefined;
  });
});

describeDevMoonbeam("Staking - Rewards Auto-Compound - 0% auto-compound", (context) => {
  before("should delegate", async () => {
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
  });

  it("should not compound reward and emit no event", async () => {
    const rewardDelay = context.polkadotApi.consts.parachainStaking.rewardPaymentDelay;
    await jumpRounds(context, rewardDelay.addn(1).toNumber());
    const blockHash = (await context.createBlock()).block.hash.toString();
    const events = await getRewardedAndCompoundedEvents(context, blockHash);
    const rewardedEvent = events.rewarded.find(({ account }) => account === ethan.address);
    const compoundedEvent = events.compounded.find(({ delegator }) => delegator === ethan.address);

    expect(rewardedEvent, "delegator was not rewarded").to.not.be.undefined;
    expect(compoundedEvent, "delegator reward was erroneously compounded").to.be.undefined;
  });
});

describeDevMoonbeam("Staking - Rewards Auto-Compound - 1% auto-compound", (context) => {
  before("should delegate", async () => {
    await expectOk(
      context.createBlock([
        context.polkadotApi.tx.sudo
          .sudo(context.polkadotApi.tx.parachainStaking.setBlocksPerRound(10))
          .signAsync(alith),
        context.polkadotApi.tx.parachainStaking
          .delegateWithAutoCompound(alith.address, MIN_GLMR_DELEGATOR, 1, 0, 0, 0)
          .signAsync(ethan),
      ])
    );
  });

  it("should compound 1% reward", async () => {
    const rewardDelay = context.polkadotApi.consts.parachainStaking.rewardPaymentDelay;
    await jumpRounds(context, rewardDelay.addn(1).toNumber());
    const blockHash = (await context.createBlock()).block.hash.toString();
    const events = await getRewardedAndCompoundedEvents(context, blockHash);
    const rewardedEvent = events.rewarded.find(({ account }) => account === ethan.address);
    const compoundedEvent = events.compounded.find(({ delegator }) => delegator === ethan.address);

    expect(rewardedEvent, "delegator was not rewarded").to.not.be.undefined;
    expect(
      compoundedEvent.amount.toString(),
      "delegator did not get 1% of their rewarded auto-compounded"
    ).to.equal(new Percent(1).ofCeil(rewardedEvent.amount).toString());
  });
});

describeDevMoonbeam("Staking - Rewards Auto-Compound - 50% auto-compound", (context) => {
  before("should delegate", async () => {
    await expectOk(
      context.createBlock([
        context.polkadotApi.tx.sudo
          .sudo(context.polkadotApi.tx.parachainStaking.setBlocksPerRound(10))
          .signAsync(alith),
        context.polkadotApi.tx.parachainStaking
          .delegateWithAutoCompound(alith.address, MIN_GLMR_DELEGATOR, 50, 0, 0, 0)
          .signAsync(ethan),
      ])
    );
  });

  it("should compound 50% reward", async () => {
    const rewardDelay = context.polkadotApi.consts.parachainStaking.rewardPaymentDelay;
    await jumpRounds(context, rewardDelay.addn(1).toNumber());
    const blockHash = (await context.createBlock()).block.hash.toString();
    const events = await getRewardedAndCompoundedEvents(context, blockHash);
    const rewardedEvent = events.rewarded.find(({ account }) => account === ethan.address);
    const compoundedEvent = events.compounded.find(({ delegator }) => delegator === ethan.address);

    expect(rewardedEvent, "delegator was not rewarded").to.not.be.undefined;
    expect(
      compoundedEvent.amount.toString(),
      "delegator did not get 50% of their rewarded auto-compounded"
    ).to.equal(new Percent(50).ofCeil(rewardedEvent.amount).toString());
  });
});

describeDevMoonbeam("Staking - Rewards Auto-Compound - 100% auto-compound", (context) => {
  before("should delegate", async () => {
    await expectOk(
      context.createBlock([
        context.polkadotApi.tx.sudo
          .sudo(context.polkadotApi.tx.parachainStaking.setBlocksPerRound(10))
          .signAsync(alith),
        context.polkadotApi.tx.parachainStaking
          .delegateWithAutoCompound(alith.address, MIN_GLMR_DELEGATOR, 100, 0, 0, 0)
          .signAsync(ethan),
      ])
    );
  });

  it("should compound 100% reward", async () => {
    const rewardDelay = context.polkadotApi.consts.parachainStaking.rewardPaymentDelay;
    await jumpRounds(context, rewardDelay.addn(1).toNumber());
    const blockHash = (await context.createBlock()).block.hash.toString();
    const events = await getRewardedAndCompoundedEvents(context, blockHash);
    const rewardedEvent = events.rewarded.find(({ account }) => account === ethan.address);
    const compoundedEvent = events.compounded.find(({ delegator }) => delegator === ethan.address);

    expect(rewardedEvent, "delegator was not rewarded").to.not.be.undefined;
    expect(
      compoundedEvent.amount.toString(),
      "delegator did not get 100% of their rewarded auto-compounded"
    ).to.equal(rewardedEvent.amount.toString());
  });
});

describeDevMoonbeam("Staking - Rewards Auto-Compound - no revoke requests", (context) => {
  before("should delegate", async () => {
    await expectOk(
      context.createBlock([
        context.polkadotApi.tx.sudo
          .sudo(context.polkadotApi.tx.parachainStaking.setBlocksPerRound(10))
          .signAsync(alith),
        context.polkadotApi.tx.parachainStaking
          .delegateWithAutoCompound(alith.address, MIN_GLMR_DELEGATOR, 100, 0, 0, 0)
          .signAsync(ethan),
      ])
    );
  });

  it("should auto-compound full amount", async () => {
    const rewardDelay = context.polkadotApi.consts.parachainStaking.rewardPaymentDelay;
    await jumpRounds(context, rewardDelay.addn(1).toNumber());
    const blockHash = (await context.createBlock()).block.hash.toString();
    const events = await getRewardedAndCompoundedEvents(context, blockHash);
    const rewardedEvent = events.rewarded.find(({ account }) => account === ethan.address);
    const compoundedEvent = events.compounded.find(({ delegator }) => delegator === ethan.address);

    expect(rewardedEvent, "delegator was not rewarded").to.not.be.undefined;
    expect(
      compoundedEvent.amount.toString(),
      "delegator did not get 100% of their rewarded auto-compounded"
    ).to.equal(rewardedEvent.amount.toString());
  });
});

describeDevMoonbeam(
  "Staking - Rewards Auto-Compound - scheduled revoke request after round snapshot",
  (context) => {
    before("should scheduleLeaveDelegators", async () => {
      await expectOk(
        context.createBlock([
          context.polkadotApi.tx.sudo
            .sudo(context.polkadotApi.tx.parachainStaking.setBlocksPerRound(10))
            .signAsync(alith),
          context.polkadotApi.tx.parachainStaking
            .delegateWithAutoCompound(alith.address, MIN_GLMR_DELEGATOR, 100, 0, 0, 0)
            .signAsync(ethan),
        ])
      );
      await jumpRounds(
        context,
        context.polkadotApi.consts.parachainStaking.rewardPaymentDelay.toNumber()
      );
      await expectOk(
        context.createBlock(
          context.polkadotApi.tx.parachainStaking
            .scheduleRevokeDelegation(alith.address)
            .signAsync(ethan)
        )
      );
    });

    it("should reward but not compound", async () => {
      await jumpRounds(context, 1);
      const blockHash = (await context.createBlock()).block.hash.toString();
      const events = await getRewardedAndCompoundedEvents(context, blockHash);
      const rewardedEvent = events.rewarded.find(({ account }) => account === ethan.address);
      const compoundedEvent = events.compounded.find(
        ({ delegator }) => delegator === ethan.address
      );

      expect(rewardedEvent, "delegator was not rewarded").to.not.be.undefined;
      expect(compoundedEvent, "delegator reward was erroneously auto-compounded").to.be.undefined;
    });
  }
);

describeDevMoonbeam("Staking - Rewards Auto-Compound - delegator leave", (context) => {
  before("should delegate and add baltathar as candidate", async () => {
    await expectOk(
      context.createBlock([
        context.polkadotApi.tx.sudo
          .sudo(context.polkadotApi.tx.parachainStaking.setBlocksPerRound(10))
          .signAsync(alith),
        context.polkadotApi.tx.parachainStaking
          .joinCandidates(MIN_GLMR_STAKING, 1)
          .signAsync(baltathar),
        context.polkadotApi.tx.parachainStaking
          .delegateWithAutoCompound(alith.address, MIN_GLMR_DELEGATOR, 100, 0, 0, 0)
          .signAsync(ethan),
      ])
    );
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .delegateWithAutoCompound(baltathar.address, MIN_GLMR_DELEGATOR, 100, 0, 0, 1)
          .signAsync(ethan)
      )
    );

    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking.scheduleLeaveDelegators().signAsync(ethan)
      )
    );

    const roundDelay = context.polkadotApi.consts.parachainStaking.leaveDelegatorsDelay.toNumber();
    await jumpRounds(context, roundDelay);
  });

  it("should remove all auto-compound configs across multiple candidates", async () => {
    const autoCompoundDelegationsAlithBefore =
      await context.polkadotApi.query.parachainStaking.autoCompoundingDelegations(alith.address);
    const autoCompoundDelegationsBaltatharBefore =
      await context.polkadotApi.query.parachainStaking.autoCompoundingDelegations(
        baltathar.address
      );
    expect(autoCompoundDelegationsAlithBefore.toJSON()).to.not.be.empty;
    expect(autoCompoundDelegationsBaltatharBefore.toJSON()).to.not.be.empty;

    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .executeLeaveDelegators(ethan.address, 2)
        .signAsync(ethan)
    );

    const autoCompoundDelegationsAlithAfter =
      await context.polkadotApi.query.parachainStaking.autoCompoundingDelegations(alith.address);
    const autoCompoundDelegationsBaltatharAfter =
      await context.polkadotApi.query.parachainStaking.autoCompoundingDelegations(
        baltathar.address
      );
    expect(autoCompoundDelegationsAlithAfter.toJSON()).to.be.empty;
    expect(autoCompoundDelegationsBaltatharAfter.toJSON()).to.be.empty;
  });
});

describeDevMoonbeam("Staking - Rewards Auto-Compound - candidate leave", (context) => {
  before("should delegate and add baltathar as candidate", async () => {
    await expectOk(
      context.createBlock([
        context.polkadotApi.tx.sudo
          .sudo(context.polkadotApi.tx.parachainStaking.setBlocksPerRound(10))
          .signAsync(alith),
        context.polkadotApi.tx.parachainStaking
          .joinCandidates(MIN_GLMR_STAKING, 1)
          .signAsync(baltathar),
        context.polkadotApi.tx.parachainStaking
          .delegateWithAutoCompound(alith.address, MIN_GLMR_DELEGATOR, 100, 0, 0, 0)
          .signAsync(ethan),
      ])
    );
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .delegateWithAutoCompound(baltathar.address, MIN_GLMR_DELEGATOR, 100, 0, 0, 1)
          .signAsync(ethan)
      )
    );

    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking.scheduleLeaveCandidates(2).signAsync(baltathar)
      )
    );

    const roundDelay = context.polkadotApi.consts.parachainStaking.leaveCandidatesDelay.toNumber();
    await jumpRounds(context, roundDelay);
  });

  it("should remove auto-compound config only for baltathar", async () => {
    const autoCompoundDelegationsAlithBefore =
      await context.polkadotApi.query.parachainStaking.autoCompoundingDelegations(alith.address);
    const autoCompoundDelegationsBaltatharBefore =
      await context.polkadotApi.query.parachainStaking.autoCompoundingDelegations(
        baltathar.address
      );
    expect(autoCompoundDelegationsAlithBefore.toJSON()).to.not.be.empty;
    expect(autoCompoundDelegationsBaltatharBefore.toJSON()).to.not.be.empty;

    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .executeLeaveCandidates(baltathar.address, 1)
        .signAsync(ethan)
    );

    const autoCompoundDelegationsAlithAfter =
      await context.polkadotApi.query.parachainStaking.autoCompoundingDelegations(alith.address);
    const autoCompoundDelegationsBaltatharAfter =
      await context.polkadotApi.query.parachainStaking.autoCompoundingDelegations(
        baltathar.address
      );
    expect(autoCompoundDelegationsAlithAfter.toJSON()).to.not.be.empty;
    expect(autoCompoundDelegationsBaltatharAfter.toJSON()).to.be.empty;
  });
});

describeDevMoonbeam("Staking - Rewards Auto-Compound - bottom delegation kick", (context) => {
  let newDelegator: KeyringPair;

  before("should delegate and add baltathar as candidate", async () => {
    const maxDelegationCount =
      context.polkadotApi.consts.parachainStaking.maxTopDelegationsPerCandidate.toNumber() +
      context.polkadotApi.consts.parachainStaking.maxBottomDelegationsPerCandidate.toNumber();
    const [delegator, ...otherDelegators] = new Array(maxDelegationCount)
      .fill(0)
      .map(() => generateKeyringPair());
    newDelegator = delegator;

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

    let alithNonce = await context.web3.eth.getTransactionCount(alith.address);
    await expectOk(
      context.createBlock([
        context.polkadotApi.tx.balances
          .transfer(newDelegator.address, MIN_GLMR_STAKING)
          .signAsync(alith, { nonce: alithNonce++ }),
        ...otherDelegators.map((d) =>
          context.polkadotApi.tx.balances
            .transfer(d.address, MIN_GLMR_STAKING)
            .signAsync(alith, { nonce: alithNonce++ })
        ),
      ])
    );

    // fill all delegations, we split this into two blocks as it will not fit into one.
    // we use a maxDelegationCount here, since the transactions can come out of order.
    await expectOk(
      context.createBlock([
        context.polkadotApi.tx.parachainStaking
          .delegate(baltathar.address, MIN_GLMR_DELEGATOR, 0, 1)
          .signAsync(ethan),
        ...otherDelegators
          .slice(0, 150)
          .map((d) =>
            context.polkadotApi.tx.parachainStaking
              .delegate(alith.address, MIN_GLMR_DELEGATOR + 10n * GLMR, maxDelegationCount, 1)
              .signAsync(d)
          ),
      ])
    );
    await expectOk(
      context.createBlock([
        ...otherDelegators
          .slice(150)
          .map((d) =>
            context.polkadotApi.tx.parachainStaking
              .delegate(alith.address, MIN_GLMR_DELEGATOR + 10n * GLMR, maxDelegationCount, 1)
              .signAsync(d)
          ),
      ])
    );

    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .setAutoCompound(alith.address, 100, 0, 2)
          .signAsync(ethan)
      )
    );
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .setAutoCompound(baltathar.address, 100, 0, 2)
          .signAsync(ethan)
      )
    );
  });

  it("should remove auto-compound config only for alith", async () => {
    const autoCompoundDelegationsAlithBefore =
      await context.polkadotApi.query.parachainStaking.autoCompoundingDelegations(alith.address);
    const autoCompoundDelegationsBaltatharBefore =
      await context.polkadotApi.query.parachainStaking.autoCompoundingDelegations(
        baltathar.address
      );
    expect(autoCompoundDelegationsAlithBefore.toJSON()).to.not.be.empty;
    expect(autoCompoundDelegationsBaltatharBefore.toJSON()).to.not.be.empty;

    const maxDelegationCount =
      context.polkadotApi.consts.parachainStaking.maxTopDelegationsPerCandidate.toNumber() +
      context.polkadotApi.consts.parachainStaking.maxBottomDelegationsPerCandidate.toNumber();

    // This kicks ethan from bottom delegations for alith
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .delegate(alith.address, MIN_GLMR_DELEGATOR + 10n * GLMR, maxDelegationCount, 0)
          .signAsync(newDelegator)
      )
    );

    const autoCompoundDelegationsAlithAfter =
      await context.polkadotApi.query.parachainStaking.autoCompoundingDelegations(alith.address);
    const autoCompoundDelegationsBaltatharAfter =
      await context.polkadotApi.query.parachainStaking.autoCompoundingDelegations(
        baltathar.address
      );

    expect(autoCompoundDelegationsAlithAfter.toJSON()).to.be.empty;
    expect(autoCompoundDelegationsBaltatharAfter.toJSON()).to.not.be.empty;
  });
});

async function getRewardedAndCompoundedEvents(context: DevTestContext, blockHash: string) {
  return (await (await context.polkadotApi.at(blockHash)).query.system.events()).reduce(
    (acc, event) => {
      if (context.polkadotApi.events.parachainStaking.Rewarded.is(event.event)) {
        acc.rewarded.push({
          account: event.event.data[0].toString(),
          amount: event.event.data[1],
        });
      } else if (context.polkadotApi.events.parachainStaking.Compounded.is(event.event)) {
        acc.compounded.push({
          candidate: event.event.data[0].toString(),
          delegator: event.event.data[1].toString(),
          amount: event.event.data[2],
        });
      }
      return acc;
    },
    { rewarded: [], compounded: [] }
  );
}

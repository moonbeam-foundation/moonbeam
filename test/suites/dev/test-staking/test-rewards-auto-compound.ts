import "@moonbeam-network/api-augment";
import { DevModeContext, beforeAll, describeSuite, expect } from "@moonwall/cli";
import {
  GLMR,
  KeyringPair,
  MIN_GLMR_DELEGATOR,
  MIN_GLMR_STAKING,
  Percent,
  alith,
  baltathar,
  ethan,
  generateKeyringPair,
} from "@moonwall/util";
import { jumpRounds } from "../../../helpers/block.js";
import { chunk } from "../../../../tests/util/common.js";

describeSuite({
  id: "D2954",
  title: "Staking - Rewards Auto-Compound - no auto-compound config",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      await context.createBlock(
        [
          context
            .polkadotJs()
            .tx.sudo.sudo(context.polkadotJs().tx.parachainStaking.setBlocksPerRound(10))
            .signAsync(alith),
          context
            .polkadotJs()
            .tx.parachainStaking.delegate(alith.address, MIN_GLMR_DELEGATOR, 0, 0)
            .signAsync(ethan),
        ],
        { allowFailures: false }
      );
    });

    it({
      id: "T01",
      title: "should not compound reward and emit no event",
      test: async () => {
        const rewardDelay = context.polkadotJs().consts.parachainStaking.rewardPaymentDelay;
        await jumpRounds(context, rewardDelay.addn(1).toNumber());
        const blockHash = (await context.createBlock()).block.hash.toString();
        const events = await getRewardedAndCompoundedEvents(context, blockHash);
        const rewardedEvent = events.rewarded.find(({ account }: any) => account === ethan.address);
        const compoundedEvent = events.compounded.find(
          ({ delegator }: any) => delegator === ethan.address
        );

        expect(rewardedEvent, "delegator was not rewarded").to.not.be.undefined;
        expect(compoundedEvent, "delegator reward was erroneously compounded").to.be.undefined;
      },
    });
  },
});

describeSuite({
  id: "",
  title: "Staking - Rewards Auto-Compound - 0% auto-compound",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      await context.createBlock([
        context
          .polkadotJs()
          .tx.sudo.sudo(context.polkadotJs().tx.parachainStaking.setBlocksPerRound(10))
          .signAsync(alith),
        context
          .polkadotJs()
          .tx.parachainStaking.delegate(alith.address, MIN_GLMR_DELEGATOR, 0, 0)
          .signAsync(ethan),
      ]);
    });

    it({
      id: "T01",
      title: "should not compound reward and emit no event",
      test: async () => {
        const rewardDelay = context.polkadotJs().consts.parachainStaking.rewardPaymentDelay;
        await jumpRounds(context, rewardDelay.addn(1).toNumber());
        const blockHash = (await context.createBlock()).block.hash.toString();
        const events = await getRewardedAndCompoundedEvents(context, blockHash);
        const rewardedEvent = events.rewarded.find(({ account }: any) => account === ethan.address);
        const compoundedEvent = events.compounded.find(
          ({ delegator }: any) => delegator === ethan.address
        );

        expect(rewardedEvent, "delegator was not rewarded").to.not.be.undefined;
        expect(compoundedEvent, "delegator reward was erroneously compounded").to.be.undefined;
      },
    });
  },
});

describeSuite({
  id: "",
  title: "Staking - Rewards Auto-Compound - 1% auto-compound",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      await context.createBlock([
        context
          .polkadotJs()
          .tx.sudo.sudo(context.polkadotJs().tx.parachainStaking.setBlocksPerRound(10))
          .signAsync(alith),
        context
          .polkadotJs()
          .tx.parachainStaking.delegateWithAutoCompound(
            alith.address,
            MIN_GLMR_DELEGATOR,
            1,
            0,
            0,
            0
          )
          .signAsync(ethan),
      ]);
    });

    it({
      id: "",
      title: "should compound 1% reward",
      test: async () => {
        const rewardDelay = context.polkadotJs().consts.parachainStaking.rewardPaymentDelay;
        await jumpRounds(context, rewardDelay.addn(1).toNumber());
        const blockHash = (await context.createBlock()).block.hash.toString();
        const events = await getRewardedAndCompoundedEvents(context, blockHash);
        const rewardedEvent = events.rewarded.find(({ account }: any) => account === ethan.address);
        const compoundedEvent = events.compounded.find(
          ({ delegator }: any) => delegator === ethan.address
        );

        expect(rewardedEvent, "delegator was not rewarded").to.not.be.undefined;
        expect(
          compoundedEvent.amount.toString(),
          "delegator did not get 1% of their rewarded auto-compounded"
        ).to.equal(new Percent(1).ofCeil(rewardedEvent.amount).toString());
      },
    });
  },
});

describeSuite({
  id: "",
  title: "Staking - Rewards Auto-Compound - 50% auto-compound",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      await context.createBlock(
        [
          context
            .polkadotJs()
            .tx.sudo.sudo(context.polkadotJs().tx.parachainStaking.setBlocksPerRound(10))
            .signAsync(alith),
          context
            .polkadotJs()
            .tx.parachainStaking.delegateWithAutoCompound(
              alith.address,
              MIN_GLMR_DELEGATOR,
              50,
              0,
              0,
              0
            )
            .signAsync(ethan),
        ],
        { allowFailures: false }
      );
    });

    it({
      id: "T01",
      title: "should compound 50% reward",
      test: async () => {
        const rewardDelay = context.polkadotJs().consts.parachainStaking.rewardPaymentDelay;
        await jumpRounds(context, rewardDelay.addn(1).toNumber());
        const blockHash = (await context.createBlock()).block.hash.toString();
        const events = await getRewardedAndCompoundedEvents(context, blockHash);
        const rewardedEvent = events.rewarded.find(({ account }: any) => account === ethan.address);
        const compoundedEvent = events.compounded.find(
          ({ delegator }: any) => delegator === ethan.address
        );

        expect(rewardedEvent, "delegator was not rewarded").to.not.be.undefined;
        expect(
          compoundedEvent.amount.toString(),
          "delegator did not get 50% of their rewarded auto-compounded"
        ).to.equal(new Percent(50).ofCeil(rewardedEvent.amount).toString());
      },
    });
  },
});

describeSuite({
  id: "",
  title: "Staking - Rewards Auto-Compound - 100% auto-compound",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      await context.createBlock(
        [
          context
            .polkadotJs()
            .tx.sudo.sudo(context.polkadotJs().tx.parachainStaking.setBlocksPerRound(10))
            .signAsync(alith),
          context
            .polkadotJs()
            .tx.parachainStaking.delegateWithAutoCompound(
              alith.address,
              MIN_GLMR_DELEGATOR,
              100,
              0,
              0,
              0
            )
            .signAsync(ethan),
        ],
        { allowFailures: false }
      );
    });

    it({
      id: "T01",
      title: "should compound 100% reward",
      test: async () => {
        const rewardDelay = context.polkadotJs().consts.parachainStaking.rewardPaymentDelay;
        await jumpRounds(context, rewardDelay.addn(1).toNumber());
        const blockHash = (await context.createBlock()).block.hash.toString();
        const events = await getRewardedAndCompoundedEvents(context, blockHash);
        const rewardedEvent = events.rewarded.find(({ account }: any) => account === ethan.address);
        const compoundedEvent = events.compounded.find(
          ({ delegator }: any) => delegator === ethan.address
        );

        expect(rewardedEvent, "delegator was not rewarded").to.not.be.undefined;
        expect(
          compoundedEvent.amount.toString(),
          "delegator did not get 100% of their rewarded auto-compounded"
        ).to.equal(rewardedEvent.amount.toString());
      },
    });
  },
});

describeSuite({
  id: "",
  title: "Staking - Rewards Auto-Compound - no revoke requests",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      await context.createBlock(
        [
          context
            .polkadotJs()
            .tx.sudo.sudo(context.polkadotJs().tx.parachainStaking.setBlocksPerRound(10))
            .signAsync(alith),
          context
            .polkadotJs()
            .tx.parachainStaking.delegateWithAutoCompound(
              alith.address,
              MIN_GLMR_DELEGATOR,
              100,
              0,
              0,
              0
            )
            .signAsync(ethan),
        ],
        { allowFailures: false }
      );
    });

    it({
      id: "T01",
      title: "should auto-compound full amount",
      test: async () => {
        const rewardDelay = context.polkadotJs().consts.parachainStaking.rewardPaymentDelay;
        await jumpRounds(context, rewardDelay.addn(1).toNumber());
        const blockHash = (await context.createBlock()).block.hash.toString();
        const events = await getRewardedAndCompoundedEvents(context, blockHash);
        const rewardedEvent = events.rewarded.find(({ account }: any) => account === ethan.address);
        const compoundedEvent = events.compounded.find(
          ({ delegator }: any) => delegator === ethan.address
        );

        expect(rewardedEvent, "delegator was not rewarded").to.not.be.undefined;
        expect(
          compoundedEvent!.amount.toString(),
          "delegator did not get 100% of their rewarded auto-compounded"
        ).to.equal(rewardedEvent!.amount.toString());
      },
    });
  },
});

describeSuite({
  id: "",
  title: "Staking - Rewards Auto-Compound - scheduled revoke request after round snapshot",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      await context.createBlock(
        [
          context
            .polkadotJs()
            .tx.sudo.sudo(context.polkadotJs().tx.parachainStaking.setBlocksPerRound(10))
            .signAsync(alith),
          context
            .polkadotJs()
            .tx.parachainStaking.delegateWithAutoCompound(
              alith.address,
              MIN_GLMR_DELEGATOR,
              100,
              0,
              0,
              0
            )
            .signAsync(ethan),
        ],
        { allowFailures: false }
      );
      await jumpRounds(
        context,
        context.polkadotJs().consts.parachainStaking.rewardPaymentDelay.toNumber()
      );
      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.scheduleRevokeDelegation(alith.address)
          .signAsync(ethan),
        { allowFailures: false }
      );
    });

    it({
      id: "T01",
      title: "should reward but not compound",
      test: async () => {
        await jumpRounds(context, 1);
        const blockHash = (await context.createBlock()).block.hash.toString();
        const events = await getRewardedAndCompoundedEvents(context, blockHash);
        const rewardedEvent = events.rewarded.find(({ account }: any) => account === ethan.address);
        const compoundedEvent = events.compounded.find(
          ({ delegator }: any) => delegator === ethan.address
        );

        expect(rewardedEvent, "delegator was not rewarded").to.not.be.undefined;
        expect(compoundedEvent, "delegator reward was erroneously auto-compounded").to.be.undefined;
      },
    });
  },
});

describeSuite({
  id: "",
  title: "Staking - Rewards Auto-Compound - delegator leave",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      await context.createBlock(
        [
          context
            .polkadotJs()
            .tx.sudo.sudo(context.polkadotJs().tx.parachainStaking.setBlocksPerRound(10))
            .signAsync(alith),
          context
            .polkadotJs()
            .tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, 1)
            .signAsync(baltathar),
          context
            .polkadotJs()
            .tx.parachainStaking.delegateWithAutoCompound(
              alith.address,
              MIN_GLMR_DELEGATOR,
              100,
              0,
              0,
              0
            )
            .signAsync(ethan),
        ],
        { allowFailures: false }
      );
      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.delegateWithAutoCompound(
            baltathar.address,
            MIN_GLMR_DELEGATOR,
            100,
            0,
            0,
            1
          )
          .signAsync(ethan),
        { allowFailures: false }
      );

      await context.createBlock(
        context.polkadotJs().tx.parachainStaking.scheduleLeaveDelegators().signAsync(ethan),
        { allowFailures: false }
      );
      const roundDelay = context
        .polkadotJs()
        .consts.parachainStaking.leaveDelegatorsDelay.toNumber();
      await jumpRounds(context, roundDelay);
    });

    it({
      id: "T01",
      title: "should remove all auto-compound configs across multiple candidates",
      test: async () => {
        const autoCompoundDelegationsAlithBefore = await context
          .polkadotJs()
          .query.parachainStaking.autoCompoundingDelegations(alith.address);
        const autoCompoundDelegationsBaltatharBefore = await context
          .polkadotJs()
          .query.parachainStaking.autoCompoundingDelegations(baltathar.address);
        expect(autoCompoundDelegationsAlithBefore.toJSON()).to.not.be.empty;
        expect(autoCompoundDelegationsBaltatharBefore.toJSON()).to.not.be.empty;

        await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.executeLeaveDelegators(ethan.address, 2)
            .signAsync(ethan)
        );

        const autoCompoundDelegationsAlithAfter = await context
          .polkadotJs()
          .query.parachainStaking.autoCompoundingDelegations(alith.address);
        const autoCompoundDelegationsBaltatharAfter = await context
          .polkadotJs()
          .query.parachainStaking.autoCompoundingDelegations(baltathar.address);
        expect(autoCompoundDelegationsAlithAfter.toJSON()).to.be.empty;
        expect(autoCompoundDelegationsBaltatharAfter.toJSON()).to.be.empty;
      },
    });
  },
});

describeSuite({
  id: "",
  title: "Staking - Rewards Auto-Compound - candidate leave",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      await context.createBlock(
        [
          context
            .polkadotJs()
            .tx.sudo.sudo(context.polkadotJs().tx.parachainStaking.setBlocksPerRound(10))
            .signAsync(alith),
          context
            .polkadotJs()
            .tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, 1)
            .signAsync(baltathar),
          context
            .polkadotJs()
            .tx.parachainStaking.delegateWithAutoCompound(
              alith.address,
              MIN_GLMR_DELEGATOR,
              100,
              0,
              0,
              0
            )
            .signAsync(ethan),
        ],
        { allowFailures: false }
      );
      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.delegateWithAutoCompound(
            baltathar.address,
            MIN_GLMR_DELEGATOR,
            100,
            0,
            0,
            1
          )
          .signAsync(ethan),
        { allowFailures: false }
      );

      await context.createBlock(
        context.polkadotJs().tx.parachainStaking.scheduleLeaveCandidates(2).signAsync(baltathar),
        { allowFailures: false }
      );

      const roundDelay = context
        .polkadotJs()
        .consts.parachainStaking.leaveCandidatesDelay.toNumber();
      await jumpRounds(context, roundDelay);
    });

    it({
      id: "T01",
      title: "should remove auto-compound config only for baltathar",
      test: async () => {
        const autoCompoundDelegationsAlithBefore = await context
          .polkadotJs()
          .query.parachainStaking.autoCompoundingDelegations(alith.address);
        const autoCompoundDelegationsBaltatharBefore = await context
          .polkadotJs()
          .query.parachainStaking.autoCompoundingDelegations(baltathar.address);
        expect(autoCompoundDelegationsAlithBefore.toJSON()).to.not.be.empty;
        expect(autoCompoundDelegationsBaltatharBefore.toJSON()).to.not.be.empty;

        await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.executeLeaveCandidates(baltathar.address, 1)
            .signAsync(ethan)
        );

        const autoCompoundDelegationsAlithAfter = await context
          .polkadotJs()
          .query.parachainStaking.autoCompoundingDelegations(alith.address);
        const autoCompoundDelegationsBaltatharAfter = await context
          .polkadotJs()
          .query.parachainStaking.autoCompoundingDelegations(baltathar.address);
        expect(autoCompoundDelegationsAlithAfter.toJSON()).to.not.be.empty;
        expect(autoCompoundDelegationsBaltatharAfter.toJSON()).to.be.empty;
      },
    });
  },
});

describeSuite({
  id: "",
  title: "Staking - Rewards Auto-Compound - bottom delegation kick",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let newDelegator: KeyringPair;

    beforeAll(async () => {
      const maxDelegationCount =
        context.polkadotJs().consts.parachainStaking.maxTopDelegationsPerCandidate.toNumber() +
        context.polkadotJs().consts.parachainStaking.maxBottomDelegationsPerCandidate.toNumber();
      const [delegator, ...otherDelegators] = new Array(maxDelegationCount)
        .fill(0)
        .map(() => generateKeyringPair());
      newDelegator = delegator;

      await context.createBlock(
        [
          context
            .polkadotJs()
            .tx.sudo.sudo(context.polkadotJs().tx.parachainStaking.setBlocksPerRound(10))
            .signAsync(alith),
          context
            .polkadotJs()
            .tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, 1)
            .signAsync(baltathar),
          context
            .polkadotJs()
            .tx.parachainStaking.delegate(alith.address, MIN_GLMR_DELEGATOR, 0, 0)
            .signAsync(ethan),
        ],
        { allowFailures: false }
      );

      let alithNonce = await context
        .viem()
        .getTransactionCount({ address: alith.address as `0x${string}` });
      await context.createBlock(
        [
          context
            .polkadotJs()
            .tx.balances.transfer(newDelegator.address, MIN_GLMR_STAKING)
            .signAsync(alith, { nonce: alithNonce++ }),
          ...otherDelegators.map((d) =>
            context
              .polkadotJs()
              .tx.balances.transfer(d.address, MIN_GLMR_STAKING)
              .signAsync(alith, { nonce: alithNonce++ })
          ),
        ],
        { allowFailures: false }
      );

      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.delegate(baltathar.address, MIN_GLMR_DELEGATOR, 0, 1)
          .signAsync(ethan),
        { allowFailures: false }
      );

      // fill all delegations, we split this into multiple blocks as it will not fit into one.
      // we use a maxDelegationCount here, since the transactions can come out of order.
      for (const delChunk of chunk(otherDelegators, 8)) {
        await context.createBlock(
          delChunk.map((d) =>
            context
              .polkadotJs()
              .tx.parachainStaking.delegate(
                alith.address,
                MIN_GLMR_DELEGATOR + 10n * GLMR,
                maxDelegationCount,
                1
              )
              .signAsync(d)
          )
        );
      }

      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.setAutoCompound(alith.address, 100, 0, 2)
          .signAsync(ethan),
        { allowFailures: false }
      );
      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.setAutoCompound(baltathar.address, 100, 0, 2)
          .signAsync(ethan),
        { allowFailures: false }
      );
    });

    it({
      id: "T01",
      title: "should remove auto-compound config only for alith",
      test: async () => {
        const autoCompoundDelegationsAlithBefore = await context
          .polkadotJs()
          .query.parachainStaking.autoCompoundingDelegations(alith.address);
        const autoCompoundDelegationsBaltatharBefore = await context
          .polkadotJs()
          .query.parachainStaking.autoCompoundingDelegations(baltathar.address);
        expect(autoCompoundDelegationsAlithBefore.toJSON()).to.not.be.empty;
        expect(autoCompoundDelegationsBaltatharBefore.toJSON()).to.not.be.empty;

        const maxDelegationCount =
          context.polkadotJs().consts.parachainStaking.maxTopDelegationsPerCandidate.toNumber() +
          context.polkadotJs().consts.parachainStaking.maxBottomDelegationsPerCandidate.toNumber();

        // This kicks ethan from bottom delegations for alith
        await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.delegate(
              alith.address,
              MIN_GLMR_DELEGATOR + 10n * GLMR,
              maxDelegationCount,
              0
            )
            .signAsync(newDelegator),
          { allowFailures: false }
        );

        const autoCompoundDelegationsAlithAfter = await context
          .polkadotJs()
          .query.parachainStaking.autoCompoundingDelegations(alith.address);
        const autoCompoundDelegationsBaltatharAfter = await context
          .polkadotJs()
          .query.parachainStaking.autoCompoundingDelegations(baltathar.address);

        expect(autoCompoundDelegationsAlithAfter.toJSON()).to.be.empty;
        expect(autoCompoundDelegationsBaltatharAfter.toJSON()).to.not.be.empty;
      },
    });
  },
});

async function getRewardedAndCompoundedEvents(context: DevModeContext, blockHash: string) {
  return (await (await context.polkadotJs().at(blockHash)).query.system.events()).reduce(
    (acc, event:any) => {
      if (context.polkadotJs().events.parachainStaking.Rewarded.is(event.event)) {
        acc.rewarded.push({
          account: event.event.data[0].toString(),
          amount: event.event.data[1],
        });
      } else if (context.polkadotJs().events.parachainStaking.Compounded.is(event.event)) {
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

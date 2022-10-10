import "@polkadot/api-augment";
import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { MIN_GLMR_STAKING, MIN_GLMR_DELEGATOR } from "../../util/constants";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { alith, baltathar, charleth, ethan } from "../../util/accounts";
import { expectOk } from "../../util/expect";

describeDevMoonbeam(
  "Staking - Delegation Set Auto-Compound Rewards - delegator not exists",
  (context) => {
    it("should fail", async () => {
      const { result } = await context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .delegationSetAutoCompoundingReward(alith.address, 50, 0, 0)
          .signAsync(ethan)
      );
      expect(result.successful).to.be.false;
      expect(result.error.name).to.equal("DelegatorDNE");
    });
  }
);

describeDevMoonbeam(
  "Staking - Delegation Set Auto-Compound Rewards - delegation not exists",
  (context) => {
    before("setup delegation to alith", async () => {
      await expectOk(
        context.createBlock([
          context.polkadotApi.tx.parachainStaking
            .delegateWithAutoCompound(alith.address, MIN_GLMR_DELEGATOR, 50, 0, 0, 0)
            .signAsync(ethan),
        ])
      );
    });

    it("should fail", async () => {
      const { result } = await context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .delegationSetAutoCompoundingReward(baltathar.address, 50, 0, 1)
          .signAsync(ethan)
      );
      expect(result.successful).to.be.false;
      expect(result.error.name).to.equal("DelegationDNE");
    });
  }
);

describeDevMoonbeam(
  "Staking - Delegation Set Auto-Compound Rewards - wrong delegation hint",
  (context) => {
    before("setup candidates alith & baltathar, and delegators ethan & charleth", async () => {
      await expectOk(
        context.createBlock([
          context.polkadotApi.tx.parachainStaking
            .joinCandidates(MIN_GLMR_STAKING, 1)
            .signAsync(baltathar),
          context.polkadotApi.tx.parachainStaking
            .delegateWithAutoCompound(baltathar.address, MIN_GLMR_DELEGATOR, 50, 0, 0, 0)
            .signAsync(ethan),
        ])
      );
    });

    it("should fail", async () => {
      const block = await context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .delegationSetAutoCompoundingReward(alith.address, 50, 0, 0)
          .signAsync(ethan)
      );
      expect(block.result.successful).to.be.false;
      expect(block.result.error.name).to.equal("TooLowDelegationCountToAutoCompound");
    });
  }
);

describeDevMoonbeam(
  "Staking - Delegation Set Auto-Compound Rewards - \
  wrong candidate auto-compounding delegation hint",
  (context) => {
    before("setup candidates alith & baltathar, and delegators ethan & charleth", async () => {
      await expectOk(
        context.createBlock(
          context.polkadotApi.tx.parachainStaking
            .delegateWithAutoCompound(alith.address, MIN_GLMR_DELEGATOR, 50, 0, 0, 0)
            .signAsync(ethan)
        )
      );
    });

    it("should fail", async () => {
      const block = await context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .delegationSetAutoCompoundingReward(alith.address, 50, 0, 1)
          .signAsync(ethan)
      );
      expect(block.result.successful).to.be.false;
      expect(block.result.error.name).to.equal(
        "TooLowCandidateAutoCompoundingDelegationCountToAutoCompound"
      );
    });
  }
);

describeDevMoonbeam(
  "Staking - Delegation Set Auto-Compound Rewards - insert new config",
  (context) => {
    before("setup delegate", async () => {
      await expectOk(
        context.createBlock(
          context.polkadotApi.tx.parachainStaking
            .delegate(alith.address, MIN_GLMR_DELEGATOR, 0, 0)
            .signAsync(ethan)
        )
      );
    });

    it("should succeed", async () => {
      const autoCompoundConfigBefore = (
        (await context.polkadotApi.query.parachainStaking.autoCompoundingDelegations(
          alith.address
        )) as any
      )
        .toJSON()
        .find((d) => d.delegator === ethan.address);
      expect(autoCompoundConfigBefore).to.be.undefined;

      const { result } = await context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .delegationSetAutoCompoundingReward(alith.address, 50, 0, 1)
          .signAsync(ethan)
      );
      expect(result.successful).to.be.true;

      const autoCompoundConfigAfter = (
        (await context.polkadotApi.query.parachainStaking.autoCompoundingDelegations(
          alith.address
        )) as any
      )
        .toJSON()
        .find((d) => d.delegator === ethan.address);
      const delegationAutoCompoundEvents = result.events.reduce((acc, event) => {
        if (
          context.polkadotApi.events.parachainStaking.DelegationAutoCompoundingSet.is(event.event)
        ) {
          acc.push({
            candidate: event.event.data[0].toString(),
            delegator: event.event.data[1].toString(),
            value: event.event.data[2].toJSON(),
          });
        }
        return acc;
      }, []);

      expect(delegationAutoCompoundEvents).to.deep.equal([
        {
          candidate: alith.address,
          delegator: ethan.address,
          value: 50,
        },
      ]);
      expect(autoCompoundConfigAfter).to.deep.equal({
        delegator: ethan.address,
        value: 50,
      });
    });
  }
);

describeDevMoonbeam(
  "Staking - Delegation Set Auto-Compound Rewards - update existing config",
  (context) => {
    before("setup delegateWithAutoCompound", async () => {
      await expectOk(
        context.createBlock(
          context.polkadotApi.tx.parachainStaking
            .delegateWithAutoCompound(alith.address, MIN_GLMR_DELEGATOR, 10, 0, 0, 0)
            .signAsync(ethan)
        )
      );
    });

    it("should succeed", async () => {
      const autoCompoundConfigBefore = (
        (await context.polkadotApi.query.parachainStaking.autoCompoundingDelegations(
          alith.address
        )) as any
      )
        .toJSON()
        .find((d) => d.delegator === ethan.address);
      expect(autoCompoundConfigBefore).to.not.be.undefined;
      expect(autoCompoundConfigBefore.value).to.equal(10);

      const { result } = await context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .delegationSetAutoCompoundingReward(alith.address, 50, 1, 1)
          .signAsync(ethan)
      );
      expect(result.successful).to.be.true;

      const autoCompoundConfigAfter = (
        (await context.polkadotApi.query.parachainStaking.autoCompoundingDelegations(
          alith.address
        )) as any
      )
        .toJSON()
        .find((d) => d.delegator === ethan.address);
      const delegationAutoCompoundEvents = result.events.reduce((acc, event) => {
        if (
          context.polkadotApi.events.parachainStaking.DelegationAutoCompoundingSet.is(event.event)
        ) {
          acc.push({
            candidate: event.event.data[0].toString(),
            delegator: event.event.data[1].toString(),
            value: event.event.data[2].toJSON(),
          });
        }
        return acc;
      }, []);

      expect(delegationAutoCompoundEvents).to.deep.equal([
        {
          candidate: alith.address,
          delegator: ethan.address,
          value: 50,
        },
      ]);
      expect(autoCompoundConfigAfter).to.deep.equal({
        delegator: ethan.address,
        value: 50,
      });
    });
  }
);

describeDevMoonbeam(
  "Staking - Delegation Set Auto-Compound Rewards - remove existing config if 0% auto-compound",
  (context) => {
    before("setup delegateWithAutoCompound", async () => {
      await expectOk(
        context.createBlock(
          context.polkadotApi.tx.parachainStaking
            .delegateWithAutoCompound(alith.address, MIN_GLMR_DELEGATOR, 10, 0, 0, 0)
            .signAsync(ethan)
        )
      );
    });

    it("should succeed", async () => {
      const autoCompoundConfigBefore = (
        (await context.polkadotApi.query.parachainStaking.autoCompoundingDelegations(
          alith.address
        )) as any
      )
        .toJSON()
        .find((d) => d.delegator === ethan.address);
      expect(autoCompoundConfigBefore).to.not.be.undefined;
      expect(autoCompoundConfigBefore.value).to.equal(10);

      const { result } = await context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .delegationSetAutoCompoundingReward(alith.address, 0, 1, 1)
          .signAsync(ethan)
      );
      expect(result.successful).to.be.true;

      const autoCompoundConfigAfter = (
        (await context.polkadotApi.query.parachainStaking.autoCompoundingDelegations(
          alith.address
        )) as any
      )
        .toJSON()
        .find((d) => d.delegator === ethan.address);
      const delegationAutoCompoundEvents = result.events.reduce((acc, event) => {
        if (
          context.polkadotApi.events.parachainStaking.DelegationAutoCompoundingSet.is(event.event)
        ) {
          acc.push({
            candidate: event.event.data[0].toString(),
            delegator: event.event.data[1].toString(),
            value: event.event.data[2].toJSON(),
          });
        }
        return acc;
      }, []);

      expect(delegationAutoCompoundEvents).to.deep.equal([
        {
          candidate: alith.address,
          delegator: ethan.address,
          value: 0,
        },
      ]);
      expect(autoCompoundConfigAfter).to.be.undefined;
    });
  }
);

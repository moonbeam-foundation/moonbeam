import "@polkadot/api-augment";
import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { MIN_GLMR_STAKING } from "../../util/constants";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { alith, baltathar, ethan } from "../../util/accounts";
import { expectOk } from "../../util/expect";
import { jumpRounds } from "../../util/block";

describeDevMoonbeam("Staking - Rewards - no scheduled requests", (context) => {
  before("should delegate", async () => {
    await expectOk(
      context.createBlock([
        context.polkadotApi.tx.sudo
          .sudo(context.polkadotApi.tx.parachainStaking.setBlocksPerRound(10))
          .signAsync(alith),
        context.polkadotApi.tx.parachainStaking
          .delegate(alith.address, MIN_GLMR_STAKING, 0, 0)
          .signAsync(ethan),
      ])
    );
  });

  it("should reward full amount", async () => {
    const rewardDelay = context.polkadotApi.consts.parachainStaking.rewardPaymentDelay;
    await jumpRounds(context, rewardDelay.addn(1).toNumber());
    const blockHash = (await context.createBlock()).block.hash.toString();
    const allEvents = await (await context.polkadotApi.at(blockHash)).query.system.events();
    const rewardedEvents = allEvents.reduce((acc, event) => {
      if (context.polkadotApi.events.parachainStaking.Rewarded.is(event.event)) {
        acc.push({
          account: event.event.data[0].toString(),
          amount: event.event.data[1],
        });
      }
      return acc;
    }, []);

    expect(
      rewardedEvents.some(({ account }) => account == ethan.address),
      "delegator was not rewarded"
    ).to.be.true;
  });
});

/* describeDevMoonbeam(
  "Substrate Length Fees - Transaction (Moonriver)",
  (context) => {
    it("should have expensive runtime-upgrade fees", async () => {
      const fee = await testRuntimeUpgrade(context);
      expect(fee).to.equal(9_226_801_765_723_667_008n);
    });
  },
  "Legacy",
  "moonriver"
); */

describeDevMoonbeam("Staking - Rewards - scheduled leave request", (context) => {
  before("should scheduleLeaveDelegators", async () => {
    await expectOk(
      context.createBlock([
        context.polkadotApi.tx.sudo
          .sudo(context.polkadotApi.tx.parachainStaking.setBlocksPerRound(10))
          .signAsync(alith),
        context.polkadotApi.tx.parachainStaking
          .delegate(alith.address, MIN_GLMR_STAKING, 0, 0)
          .signAsync(ethan),
      ])
    );
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking.scheduleLeaveDelegators().signAsync(ethan)
      )
    );
  });

  it("should not reward", async () => {
    const rewardDelay = context.polkadotApi.consts.parachainStaking.rewardPaymentDelay;
    await jumpRounds(context, rewardDelay.addn(1).toNumber());
    const blockHash = (await context.createBlock()).block.hash.toString();
    const allEvents = await (await context.polkadotApi.at(blockHash)).query.system.events();
    const rewardedEvents = allEvents.reduce((acc, event) => {
      if (context.polkadotApi.events.parachainStaking.Rewarded.is(event.event)) {
        acc.push({
          account: event.event.data[0].toString(),
          amount: event.event.data[1],
        });
      }
      return acc;
    }, []);

    expect(
      rewardedEvents.some(({ account }) => account == ethan.address),
      "delegator was incorrectly rewarded"
    ).to.be.false;
  });
});

describeDevMoonbeam("Staking - Rewards - scheduled revoke request", (context) => {
  before("should scheduleRevokeDelegation", async () => {
    await expectOk(
      context.createBlock([
        context.polkadotApi.tx.sudo
          .sudo(context.polkadotApi.tx.parachainStaking.setBlocksPerRound(10))
          .signAsync(alith),
        context.polkadotApi.tx.parachainStaking
          .delegate(alith.address, MIN_GLMR_STAKING, 0, 0)
          .signAsync(ethan),
      ])
    );

    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .scheduleRevokeDelegation(alith.address)
          .signAsync(ethan)
      )
    );
  });

  it("should not reward", async () => {
    const rewardDelay = context.polkadotApi.consts.parachainStaking.rewardPaymentDelay;
    await jumpRounds(context, rewardDelay.addn(1).toNumber());
    const blockHash = (await context.createBlock()).block.hash.toString();
    const allEvents = await (await context.polkadotApi.at(blockHash)).query.system.events();
    const rewardedEvents = allEvents.reduce((acc, event) => {
      if (context.polkadotApi.events.parachainStaking.Rewarded.is(event.event)) {
        acc.push({
          account: event.event.data[0].toString(),
          amount: event.event.data[1],
        });
      }
      return acc;
    }, []);

    expect(
      rewardedEvents.some(({ account }) => account == ethan.address),
      "delegator was incorrectly rewarded"
    ).to.be.false;
  });
});

describeDevMoonbeam("Staking - Rewards - scheduled bond decrease request", (context) => {
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
        context.polkadotApi.tx.parachainStaking
          .delegate(alith.address, BOND_AMOUNT, 1, 0)
          .signAsync(baltathar),
      ])
    );

    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .scheduleDelegatorBondLess(alith.address, EXTRA_BOND_AMOUNT)
          .signAsync(ethan)
      )
    );
  });

  it("should reward less than baltathar", async () => {
    const rewardDelay = context.polkadotApi.consts.parachainStaking.rewardPaymentDelay;
    await jumpRounds(context, rewardDelay.addn(1).toNumber());
    const blockHash = (await context.createBlock()).block.hash.toString();
    const allEvents = await (await context.polkadotApi.at(blockHash)).query.system.events();
    const rewardedEvents = allEvents.reduce((acc, event) => {
      if (context.polkadotApi.events.parachainStaking.Rewarded.is(event.event)) {
        acc.push({
          account: event.event.data[0].toString(),
          amount: event.event.data[1],
        });
      }
      return acc;
    }, []);

    let rewardedEthan = rewardedEvents.find(({ account }) => account == ethan.address);
    let rewardedBalathar = rewardedEvents.find(({ account }) => account == baltathar.address);
    expect(rewardedEthan).is.not.undefined;
    expect(rewardedBalathar).is.not.undefined;
    expect(
      rewardedBalathar.amount.gt(rewardedEthan.amount),
      `Ethan's reward ${rewardedEthan.amount} was not less than Balathar's \
      reward ${rewardedBalathar.amount}`
    ).is.true;
  });
});

describeDevMoonbeam("Staking - Mark offline a collator not producing blocks", (context) => {
  const EXTRA_BOND_AMOUNT = 1_000_000_000_000_000_000n;
  const BOND_AMOUNT = MIN_GLMR_STAKING + EXTRA_BOND_AMOUNT;
  
  before("should test", async () => {
    await expectOk(
      context.createBlock([
        context.polkadotApi.tx.sudo
          .sudo(context.polkadotApi.tx.parachainStaking.setBlocksPerRound(10))
          .signAsync(alith)
      ])
    );

    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, 1).signAsync(ethan),
        
      )
    ); 

  });

  it("test collator offline", async () => {

    const rewardDelay = context.polkadotApi.consts.parachainStaking.rewardPaymentDelay;
    await jumpRounds(context, rewardDelay.addn(1).toNumber());
    const offlineEvents = await offline_helper(context, 100);
    

    console.log("Offline Event: ", offlineEvents);
    
  });
});

async function offline_helper(context, num_blocks: number) {
  for(let i = 0; i<num_blocks; i++){
    const blockHash = (await context.createBlock(
      //context.polkadotApi.tx.parachainStaking.goOffline().signAsync(ethan)
    )).block.hash.toString();
    const allEvents = await (await context.polkadotApi.at(blockHash)).query.system.events();
    const offlineEvents = allEvents.reduce((acc, event) => {
      if (context.polkadotApi.events.parachainStaking.CandidateWentOffline.is(event.event)) {
        acc.push({
          account: event.event.data[0].toString()
        });
      }
      return acc;
    }, []);

    if (offlineEvents.length != 0){
      return offlineEvents
    }
  }
}

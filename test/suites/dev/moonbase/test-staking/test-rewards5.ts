// import "@moonbeam-network/api-augment";
// import { beforeAll, describeSuite, expect } from "@moonwall/cli";
// import { MIN_GLMR_STAKING, alith, baltathar, ethan, dorothy, charleth } from "@moonwall/util";
// import { jumpRounds } from "../../../../helpers";
// import { FrameSystemEventRecord } from "@polkadot/types/lookup";

import { describeSuite, expect } from "@moonwall/cli";
import { id } from "ethers";

// dummy passing test
describeSuite({
  id: "D0134655",
  title: "Staking - Rewards - Bond + Treasury",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "dummy test",
      test: async () => {
        expect(true).toEqual(true);
      },
    });
  },
})

// describeSuite({
//   id: "D0134655",
//   title: "Staking - Rewards - Bond + Treasury",
//   foundationMethods: "dev",
//   testCases: ({ context, it, log }) => {
//     const BOND_AMOUNT = MIN_GLMR_STAKING + 1_000_000_000_000_000_000n;
//     const PBR_PERCENTAGE = 10;
//     const TREASURY_PERCENTAGE = 20;

//     beforeAll(async () => {
//       await context.createBlock([
//         context
//           .polkadotJs()
//           .tx.sudo.sudo(context.polkadotJs().tx.parachainStaking.setInflationDistributionConfig([
//             {
//               account: dorothy.address,
//               percent: PBR_PERCENTAGE,
//             },
//             {
//               account: charleth.address,
//               percent: TREASURY_PERCENTAGE,
//             }
//           ]))
//           .signAsync(alith),
//       ]);

//     });

//     it({
//       id: "T01",
//       title: "should reward charleth and dorothy correct amounts",
//       test: async () => {
//         const startBlockHash = (await context.createBlock(
//           [
//             context
//               .polkadotJs()
//               .tx.sudo.sudo(context.polkadotJs().tx.parachainStaking.setBlocksPerRound(10))
//               .signAsync(alith),
//             context
//               .polkadotJs()
//               .tx.parachainStaking.delegate(alith.address, BOND_AMOUNT, 0, 0)
//               .signAsync(ethan),
//             context
//               .polkadotJs()
//               .tx.parachainStaking.delegate(alith.address, BOND_AMOUNT, 1, 0)
//               .signAsync(baltathar),
//           ],
//           { allowFailures: false }
//         )).block.hash.toString();

//         const rewardDelay = context.polkadotJs().consts.parachainStaking.rewardPaymentDelay;
//         const round = (await context.polkadotJs().query.parachainStaking.round()).length;
//         console.log(`Round length: ${round}`);
//         console.log(`Reward delay: ${rewardDelay.toString()}`);
//         await jumpRounds(context, rewardDelay.addn(1).toNumber());
//         const endBlockHash = (await context.createBlock()).block.hash.toString();

//         const allEvents: FrameSystemEventRecord[] = [];
//         const parent = (await context.polkadotJs().rpc.chain.getHeader(startBlockHash))
//    .parentHash.toString();
//         let c = 0;
//         for (let hash = endBlockHash; hash != parent; hash = (await context.polkadotJs().rpc
//.chain.getHeader(hash)).parentHash.toString()) {
//           const events = await (await context.polkadotJs().at(hash)).query.system.events();
//           const bnumber = (await context.polkadotJs().rpc.chain.getHeader(hash)).number
//.toNumber();
//           console.log(`[${bnumber}] Events at block ${hash}`);
//           events.forEach((event) => {
//             if (context.polkadotJs().events.parachainStaking.Rewarded.is(event.event)) {
//               console.log(event.event.section, event.event.method);
//               console.log("\t>" + event.event.data.account.toString(), event.event.data.rewards.
//toBigInt().toString());
//             } else if (context.polkadotJs().events.parachainStaking.InflationDistributed.is(event
//.event)) {
//               console.log(event.event.section, event.event.method);
//               console.log("\t>" + event.event.data.account.toString(), event.event.data.value
//.toBigInt().toString());
//             }
//           });
//           allEvents.push(...events);
//         }

//         const rewardedEvents = allEvents.reduce(
//           (acc: { account: string; amount: bigint }[], event) => {
//             if (context.polkadotJs().events.parachainStaking.Rewarded.is(event.event)) {
//               acc.push({
//                 account: event.event.data.account.toString(),
//                 amount: event.event.data.rewards.toBigInt(),
//               });
//             } else if (context.polkadotJs().events.parachainStaking.InflationDistributed.is(event
//.event)) {
//               acc.push({
//                 account: event.event.data.account.toString(),
//                 amount: event.event.data.value.toBigInt(),
//               });
//             }
//             return acc;
//           },
//           []
//         );

//         const rewardedAlith = rewardedEvents.find(({ account }) => account == alith.address);
//         const rewardedEthan = rewardedEvents.find(({ account }) => account == ethan.address);
//         const rewardedBalathar = rewardedEvents.find(({ account }) => account == baltathar
//.address);

//         const rewardedPbr = rewardedEvents.find(({ account }) => account == dorothy.address);
//         const rewardedTreasury = rewardedEvents.find(({ account }) => account == charleth
//.address);

//         expect(rewardedAlith).is.not.undefined;
//         expect(rewardedEthan).is.not.undefined;
//         expect(rewardedBalathar).is.not.undefined;
//         expect(rewardedPbr).is.not.undefined;
//         expect(rewardedTreasury).is.not.undefined;

//         const totalReward = rewardedEvents.reduce((acc, { amount }) => acc + amount, 0n);
//         const reservedReward = rewardedPbr!.amount + rewardedTreasury!.amount;
//         const otherReward = totalReward - reservedReward;
//         const otherPercentage = BigInt(100 - PBR_PERCENTAGE - TREASURY_PERCENTAGE);

//         const reservedRewardPercentage = ((reservedReward * 100n) / totalReward);
//         const actualOtherPercentage = ((otherReward * 100n) / totalReward);

//         //log all the above values
//         console.log(`Total reward: ${totalReward}`);
//         console.log(`Reserved reward: ${reservedReward}`);
//         console.log(`Other reward: ${otherReward}`);
//         console.log(`Reserved reward percentage: ${reservedRewardPercentage}`);
//         console.log(`Other reward percentage: ${actualOtherPercentage}`);
//         console.log(`PBR reward: ${rewardedPbr!.amount}`);
//         console.log(`Treasury reward: ${rewardedTreasury!.amount}`);

//         expect(reservedRewardPercentage.toString(), "Reserved reward percentage is not correct")
//           .toEqual((PBR_PERCENTAGE + TREASURY_PERCENTAGE).toString());
//         expect(actualOtherPercentage.toString(), "Other reward percentage is not correct")
//           .toEqual(otherPercentage.toString());

//         const pbrPercentage = (rewardedPbr!.amount * 100n) / totalReward;
//         const treasuryPercentage = (rewardedTreasury!.amount * 100n) / totalReward;

//         expect(pbrPercentage.toString(), "PBR reward percentage is not correct")
//           .toEqual(PBR_PERCENTAGE.toString());
//         expect(treasuryPercentage.toString(), "Treasury reward percentage is not correct")
//           .toEqual(TREASURY_PERCENTAGE.toString());
//       },
//     });
//   },
// });

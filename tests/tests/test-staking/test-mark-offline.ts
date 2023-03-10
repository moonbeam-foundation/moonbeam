/* import "@polkadot/api-augment";
import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { MIN_GLMR_STAKING } from "../../util/constants";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { alith, baltathar, ethan } from "../../util/accounts";
import { expectOk } from "../../util/expect";
import { jumpRounds } from "../../util/block";

//TODO: finish this file
describeDevMoonbeam("Staking - Mark offline a collator not producing blocks", (context) => {
  const EXTRA_BOND_AMOUNT = 1_000_000_000_000_000_000n;
  const BOND_AMOUNT = MIN_GLMR_STAKING + EXTRA_BOND_AMOUNT;

  before("should work", async () => {
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

  });

  it("test collator offline", async () => {

    const candidateState = (
      await context.polkadotApi.query.parachainStaking.candidateInfo(ethan.address)
    ).unwrap(); 


    //ethan leaves candidates pool
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking.scheduleLeaveCandidates(2).signAsync(ethan)
      )
    ); 

    const leaveDelay = context.polkadotApi.consts.parachainStaking.leaveDelegatorsDelay;
    await jumpRounds(context, leaveDelay.addn(1).toNumber());

    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking
        .executeLeaveCandidates(ethan.address, 0)
        .signAsync(ethan)
      )
    ); 

    //jump 6 rounds
    //ethan will not have produced blocks in the skipped rounds
    //await jumpRounds(context, 6);

    await context.createBlock();

    //ethan joins candidates pool again
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, 1).signAsync(ethan),
      )
    );  
     

    //check ethan went offline
    const offlineEvents = await events_helper(context, 50);

    console.log("Offline Event: ", offlineEvents);

  });
});

async function events_helper(context, num_blocks: number) {
  for(let i = 0; i<num_blocks; i++){
    const blockHash = (await context.createBlock(
      //context.polkadotApi.tx.parachainStaking.goOffline().signAsync(ethan)
    )).block.hash.toString();
    const block = await context.web3.eth.getBlock("latest");
    const allEvents = await (await context.polkadotApi.at(blockHash)).query.system.events();
    const offlineEvents = allEvents.reduce((acc, event) => {
      if (context.polkadotApi.events.parachainStaking.NewRound.is(event.event)) {
        acc.push({
          starting_block: event.event.data[0].toString(),
          round: event.event.data[1].toString(),
          selected_collators_number: event.event.data[2].toString(),
          total_balance: event.event.data[3].toString(),
          author: block.miner
        });
      }
      return acc;
    }, []);

    if (offlineEvents.length != 0){
      return offlineEvents
    }
  }
} */
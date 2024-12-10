import "@moonbeam-network/api-augment";
import type { DevModeContext } from "@moonwall/cli";

export async function getRewardedAndCompoundedEvents(context: DevModeContext, blockHash: string) {
  return (
    await (await context.polkadotJs().at(blockHash)).query.system.events()
  ).reduce<StakingEvents>(
    (acc, event: any) => {
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

interface StakingEvents {
  rewarded: { account: string; amount: any }[];
  compounded: { candidate: string; delegator: string; amount: any }[];
}

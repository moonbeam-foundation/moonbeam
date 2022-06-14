import { DevTestContext } from "../../util/setup-dev-tests";

export async function jumpToRound(context: DevTestContext, round: Number): Promise<string | null> {
  let lastBlockHash = null;
  while (true) {
    const currentRound = (
      await context.polkadotApi.query.parachainStaking.round()
    ).current.toNumber();
    if (currentRound == round) {
      return lastBlockHash;
    } else if (currentRound > round) {
      return null;
    }

    lastBlockHash = (await context.createBlock()).block.hash.toString();
  }
}

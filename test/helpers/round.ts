import "@moonbeam-network/api-augment";
import { ApiPromise } from "@polkadot/api";
import { BN } from "@polkadot/util";

/*
 * Get any block of a given round.
 * You can accelerate by given a block number close (but higher) than the expected round
 */
export const getAnyBlockOfRound = async (api: ApiPromise, roundNumber: BN, currentBlock?: BN) => {
  let iterOriginalRoundBlock = currentBlock || (await api.rpc.chain.getHeader()).number.toBn();
  for (;;) {
    const blockHash = await api.rpc.chain.getBlockHash(iterOriginalRoundBlock);
    const round = await (await api.at(blockHash)).query.parachainStaking.round();
    if (round.current.lt(roundNumber)) {
      throw new Error("Couldn't find the block for the given round");
    } else if (
      round.current.eq(roundNumber) ||
      iterOriginalRoundBlock.sub(round.length).toNumber() < 0
    ) {
      break;
    }

    // Go to previous round
    iterOriginalRoundBlock = iterOriginalRoundBlock.sub(round.length);
  }
  return iterOriginalRoundBlock;
};

import "@moonbeam-network/api-augment";
import { ApiPromise } from "@polkadot/api";
import { PalletParachainStakingRoundInfo } from "@polkadot/types/lookup";
import { BN, BN_ONE, BN_ZERO } from "@polkadot/util";

/*
 * Get any block of a given round.
 * You can accelerate by given a block number close to the expected round
 * (use latest block otherwise)
 * It is expected the blockchain always have consecutive rounds
 */
export const getRoundAt = async (api: ApiPromise, targettedRoundNumber: BN, proposedBlock?: BN) => {
  const latestRound = await (
    await api.at(await api.rpc.chain.getBlockHash())
  ).query.parachainStaking.round();
  if (targettedRoundNumber.gt(latestRound.current)) {
    throw new Error("Round number is greater than latest round");
  }

  let blockNumber = proposedBlock || (await api.rpc.chain.getHeader()).number.toBn();
  let blockHash = await api.rpc.chain.getBlockHash(blockNumber);
  let round = await (await api.at(blockHash)).query.parachainStaking.round();

  while (
    !round.current.eq(targettedRoundNumber) &&
    round.first.gt(BN_ZERO) &&
    round.first.lt(latestRound.current)
  ) {
    if (round.current.lt(targettedRoundNumber)) {
      blockNumber = round.first.add(round.length);
    } else {
      blockNumber = round.first.sub(BN_ONE);
    }
    // Go to previous round
    blockHash = await api.rpc.chain.getBlockHash(blockNumber);
    round = await (await api.at(blockHash)).query.parachainStaking.round();
  }
  return round;
};

export const getPreviousRound = async (
  api: ApiPromise,
  originRound: PalletParachainStakingRoundInfo,
  decrement: BN = BN_ONE
) => {
  const targetedRoundNumber = originRound.current.sub(decrement);
  let round = originRound;

  if (decrement.lt(BN_ZERO)) {
    throw new Error("Decrement must be positive");
  }

  if (targetedRoundNumber.lt(BN_ONE)) {
    throw new Error("Targeted round number must be at least one");
  }

  let blockNumber = originRound.first.toBn();
  let blockHash = await api.rpc.chain.getBlockHash(blockNumber);

  while (!round.current.eq(targetedRoundNumber) && round.first.gt(BN_ZERO)) {
    // Go to previous round
    blockNumber = round.first.sub(BN_ONE);
    blockHash = await api.rpc.chain.getBlockHash(blockNumber);
    round = await (await api.at(blockHash)).query.parachainStaking.round();
  }
  return round;
};

export const getNextRound = async (
  api: ApiPromise,
  originRound: PalletParachainStakingRoundInfo,
  increment: BN = BN_ONE
) => {
  const targettedRoundNumber = originRound.current.add(increment);
  let round = originRound;

  if (increment.lt(BN_ZERO)) {
    throw new Error("Increment must be positive");
  }

  const latestRound = await (
    await api.at(await api.rpc.chain.getBlockHash())
  ).query.parachainStaking.round();
  if (targettedRoundNumber.gt(latestRound.current)) {
    throw new Error("Round number is greater than latest round");
  }

  let blockNumber = originRound.first.toBn();
  let blockHash = await api.rpc.chain.getBlockHash(blockNumber);

  while (!round.current.eq(targettedRoundNumber) && round.current.lt(latestRound.current)) {
    // Go to next round
    blockNumber = round.first.add(round.length);
    blockHash = await api.rpc.chain.getBlockHash(blockNumber);
    round = await (await api.at(blockHash)).query.parachainStaking.round();
  }
  return round;
};

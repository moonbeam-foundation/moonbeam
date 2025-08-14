import "@moonbeam-network/api-augment";
import type { DevModeContext } from "@moonwall/cli";
import { VESTING_PERIOD } from "./constants.js";

export async function calculate_vested_amount(
  totalReward: bigint,
  initialPayment: bigint,
  numberOfBlocks: bigint
) {
  const amountToVest = totalReward - initialPayment;
  // On average a parachain only gets a candidate into every other relay chain block.
  // In the dev service, where the relay block number is mocked, we get exactly two relay blocks.
  const elapsedRelayBlocks = numberOfBlocks * 2n;
  const amountForBlocks = (amountToVest * elapsedRelayBlocks) / VESTING_PERIOD;
  const shouldHaveVested = initialPayment + amountForBlocks;
  return shouldHaveVested;
}

// Return the unwrapped accountsPayable or null otherwise
export const getAccountPayable = async (
  context: DevModeContext,
  address: string
): Promise<{
  totalReward: any;
  claimedReward: any;
  contributedRelayAddresses: any;
} | null> => {
  const accountsPayable = await context
    .polkadotJs()
    .query.crowdloanRewards.accountsPayable(address);
  return accountsPayable.unwrapOr(null);
};

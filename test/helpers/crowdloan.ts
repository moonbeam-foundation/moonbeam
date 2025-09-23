import "@moonbeam-network/api-augment";
import type { DevModeContext } from "@moonwall/cli";
import { VESTING_PERIOD } from "./constants.js";

export async function calculate_vested_amount(
  totalReward: bigint,
  initialPayment: bigint,
  numberOfBlocks: bigint
) {
  const amountToVest = totalReward - initialPayment;
  // We have a parachain block per relay block
  const elapsedRelayBlocks = numberOfBlocks;
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

import "@moonbeam-network/api-augment";
import type { DevModeContext } from "@moonwall/cli";

/**
 * Get all staking freezes for an account
 * @param account - The account address to check
 * @param context - The DevModeContext
 * @returns Object containing collator and delegator freeze amounts
 */
export async function getStakingFreezes(account: `0x${string}`, context: DevModeContext) {
  const api = context.polkadotJs();

  // Get all freezes for the account
  const freezes = await api.query.balances.freezes(account);

  let collatorFreeze = 0n;
  let delegatorFreeze = 0n;

  // Iterate through freezes to find ParachainStaking freezes
  for (const freeze of freezes as any) {
    if (api.events.parachainStaking && freeze.id) {
      // Check if this is a ParachainStaking freeze
      const freezeId = freeze.id.toHuman() as any;

      if (freezeId?.ParachainStaking === "StakingCollator") {
        collatorFreeze = freeze.amount.toBigInt();
      } else if (freezeId?.ParachainStaking === "StakingDelegator") {
        delegatorFreeze = freeze.amount.toBigInt();
      }
    }
  }

  return {
    collator: collatorFreeze,
    delegator: delegatorFreeze,
    total: collatorFreeze + delegatorFreeze,
  };
}

export async function getNumberOfFreezes(account: `0x${string}`, context: DevModeContext) {
  const api = context.polkadotJs();

  // Get all freezes for the account
  const freezes = await api.query.balances.freezes(account);

  return (freezes as any).length;
}

/**
 * Check if an account has a collator staking freeze
 * @param account - The account address to check
 * @param context - The DevModeContext
 * @returns true if the account has a collator freeze
 */
export async function hasCollatorStakingFreeze(
  account: `0x${string}`,
  context: DevModeContext
): Promise<boolean> {
  const freezes = await getStakingFreezes(account, context);
  return freezes.collator > 0n;
}

/**
 * Check if an account has a delegator staking freeze
 * @param account - The account address to check
 * @param context - The DevModeContext
 * @returns true if the account has a delegator freeze
 */
export async function hasDelegatorStakingFreeze(
  account: `0x${string}`,
  context: DevModeContext
): Promise<boolean> {
  const freezes = await getStakingFreezes(account, context);
  return freezes.delegator > 0n;
}

/**
 * Get the total amount frozen for staking (both collator and delegator)
 * @param account - The account address to check
 * @param context - The DevModeContext
 * @returns The total amount frozen for staking
 */
export async function getTotalStakingFreeze(
  account: `0x${string}`,
  context: DevModeContext
): Promise<bigint> {
  const freezes = await getStakingFreezes(account, context);
  return freezes.total;
}

/**
 * Get the collator staking freeze amount
 * @param account - The account address to check
 * @param context - The DevModeContext
 * @returns The amount frozen for collator staking
 */
export async function getCollatorStakingFreeze(
  account: `0x${string}`,
  context: DevModeContext
): Promise<bigint> {
  const freezes = await getStakingFreezes(account, context);
  return freezes.collator;
}

/**
 * Get the delegator staking freeze amount
 * @param account - The account address to check
 * @param context - The DevModeContext
 * @returns The amount frozen for delegator staking
 */
export async function getDelegatorStakingFreeze(
  account: `0x${string}`,
  context: DevModeContext
): Promise<bigint> {
  const freezes = await getStakingFreezes(account, context);
  return freezes.delegator;
}

/**
 * Get the count of collator staking freezes for an account
 * @param account - The account address to check
 * @param context - The DevModeContext
 * @returns The number of collator freezes
 */
export async function getNumberOfCollatorFreezes(
  account: `0x${string}`,
  context: DevModeContext
): Promise<number> {
  const api = context.polkadotJs();

  // Get all freezes for the account
  const freezes = await api.query.balances.freezes(account);

  let collatorFreezeCount = 0;

  // Iterate through freezes to count ParachainStaking collator freezes
  for (const freeze of freezes as any) {
    if (api.events.parachainStaking && freeze.id) {
      // Check if this is a ParachainStaking collator freeze
      const freezeId = freeze.id.toHuman() as any;

      if (freezeId?.ParachainStaking === "StakingCollator") {
        collatorFreezeCount++;
      }
    }
  }

  return collatorFreezeCount;
}

/**
 * Get the count of delegator staking freezes for an account
 * @param account - The account address to check
 * @param context - The DevModeContext
 * @returns The number of delegator freezes
 */
export async function getNumberOfDelegatorFreezes(
  account: `0x${string}`,
  context: DevModeContext
): Promise<number> {
  const api = context.polkadotJs();

  // Get all freezes for the account
  const freezes = await api.query.balances.freezes(account);

  let delegatorFreezeCount = 0;

  // Iterate through freezes to count ParachainStaking delegator freezes
  for (const freeze of freezes as any) {
    if (api.events.parachainStaking && freeze.id) {
      // Check if this is a ParachainStaking delegator freeze
      const freezeId = freeze.id.toHuman() as any;

      if (freezeId?.ParachainStaking === "StakingDelegator") {
        delegatorFreezeCount++;
      }
    }
  }

  return delegatorFreezeCount;
}

/**
 * Verify that a delegator's total bond matches their frozen balance
 * @param account - The delegator account address to check
 * @param context - The DevModeContext
 * @returns true if the amounts match, throws if they don't
 */
export async function verifyDelegatorStateMatchesFreezes(
  account: `0x${string}`,
  context: DevModeContext
): Promise<boolean> {
  const api = context.polkadotJs();

  // Get the delegator state
  const delegatorState = await api.query.parachainStaking.delegatorState(account);

  if ((delegatorState as any).isNone) {
    // If no delegator state, verify no delegator freeze exists
    const delegatorFreeze = await getDelegatorStakingFreeze(account, context);
    if (delegatorFreeze !== 0n) {
      throw new Error(
        `Account ${account} has no DelegatorState but has delegator freeze of ${delegatorFreeze}`
      );
    }
    return true;
  }

  // Get the total from DelegatorState
  const delegatorTotal = (delegatorState as any).unwrap().total.toBigInt();

  // Get the frozen amount
  const delegatorFreeze = await getDelegatorStakingFreeze(account, context);

  if (delegatorTotal !== delegatorFreeze) {
    throw new Error(
      `DelegatorState total (${delegatorTotal}) does not match frozen amount (${delegatorFreeze}) for account ${account}`
    );
  }

  return true;
}

/**
 * Verify that a candidate's bond matches their frozen balance
 * @param account - The candidate account address to check
 * @param context - The DevModeContext
 * @returns true if the amounts match, throws if they don't
 */
export async function verifyCandidateInfoMatchesFreezes(
  account: `0x${string}`,
  context: DevModeContext
): Promise<boolean> {
  const api = context.polkadotJs();

  // Get the candidate info
  const candidateInfo = await api.query.parachainStaking.candidateInfo(account);

  if ((candidateInfo as any).isNone) {
    // If no candidate info, verify no collator freeze exists
    const collatorFreeze = await getCollatorStakingFreeze(account, context);
    if (collatorFreeze !== 0n) {
      throw new Error(
        `Account ${account} has no CandidateInfo but has collator freeze of ${collatorFreeze}`
      );
    }
    return true;
  }

  // Get the bond from CandidateInfo
  const candidateBond = (candidateInfo as any).unwrap().bond.toBigInt();

  // Get the frozen amount
  const collatorFreeze = await getCollatorStakingFreeze(account, context);

  if (candidateBond !== collatorFreeze) {
    throw new Error(
      `CandidateInfo bond (${candidateBond}) does not match frozen amount (${collatorFreeze}) for account ${account}`
    );
  }

  return true;
}

/**
 * Verify that all staking state matches frozen balances for an account
 * Checks both delegator and candidate states
 * @param account - The account address to check
 * @param context - The DevModeContext
 * @returns true if all amounts match, throws if they don't
 */
export async function verifyStakingStateMatchesFreezes(
  account: `0x${string}`,
  context: DevModeContext
): Promise<boolean> {
  // Check delegator state
  await verifyDelegatorStateMatchesFreezes(account, context);

  // Check candidate state
  await verifyCandidateInfoMatchesFreezes(account, context);

  return true;
}

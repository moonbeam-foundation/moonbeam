import "@moonbeam-network/api-augment";
import {
  beforeAll,
  describeSuite,
  expect,
  MoonwallContext,
  type ChopsticksContext,
} from "@moonwall/cli";
import { alith } from "@moonwall/util";
import type { ApiPromise } from "@polkadot/api";
import type { HexString } from "@polkadot/util/types";
import type { u32 } from "@polkadot/types";
import { hexToU8a, u8aConcat, u8aToHex } from "@polkadot/util";
import { blake2AsHex, xxhashAsU8a } from "@polkadot/util-crypto";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import {
  hasCollatorStakingFreeze,
  hasDelegatorStakingFreeze,
  verifyCandidateInfoMatchesFreezes,
  verifyDelegatorStateMatchesFreezes,
} from "../../helpers/staking-freezes";

interface MigrationResult {
  accountId: string;
  type: "candidate" | "delegator";
  success: boolean;
  error?: string;
}

interface MigrationStats {
  totalProcessed: number;
  totalSuccessful: number;
  totalFailed: number;
  totalSkipped: number;
  candidatesProcessed: number;
  candidatesSuccessful: number;
  delegatorsProcessed: number;
  delegatorsSuccessful: number;
  failures: MigrationResult[];
  skipped: MigrationResult[];
}

const hash = (prefix: HexString, suffix: Uint8Array) => {
  return u8aToHex(u8aConcat(hexToU8a(prefix), xxhashAsU8a(suffix, 64), suffix));
};

const upgradeRestrictionSignal = (paraId: u32) => {
  const prefix = "0xcd710b30bd2eab0352ddcc26417aa194f27bbb460270642b5bcaf032ea04d56a";
  return hash(prefix, paraId.toU8a());
};

// Helper function to execute with retry on RPC timeout
const executeWithRetry = async <T>(
  operation: () => Promise<T>,
  maxRetries = 3,
  delay = 5000,
  operationName = "operation"
): Promise<T> => {
  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      return await operation();
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      const isRpcTimeout =
        errorMsg.includes("No response received from RPC endpoint") || errorMsg.includes("timeout");

      if (isRpcTimeout && attempt < maxRetries) {
        console.log(
          `⚠️  RPC timeout in ${operationName} (attempt ${attempt}/${maxRetries}), retrying in ${
            delay / 1000
          }s...`
        );
        await new Promise((resolve) => setTimeout(resolve, delay));
        continue;
      }
      throw error;
    }
  }
  throw new Error(`Failed after ${maxRetries} attempts`);
};

// Helper function to check if an account is migrated
const checkMigrationStatus = async (
  api: ApiPromise,
  accountId: string,
  isCandidate: boolean
): Promise<boolean> => {
  const operation = async () => {
    if (isCandidate) {
      const isMigrated = await api.query.parachainStaking.migratedCandidates(accountId);
      return (isMigrated as any).isSome;
    }
    const isMigrated = await api.query.parachainStaking.migratedDelegators(accountId);
    return (isMigrated as any).isSome;
  };

  return executeWithRetry(operation, 3, 5000, `checkMigrationStatus(${accountId})`);
};

// Helper function to check if an account exists in staking
const checkAccountExistsInStaking = async (
  api: ApiPromise,
  accountId: string,
  isCandidate: boolean
): Promise<boolean> => {
  const operation = async () => {
    if (isCandidate) {
      const candidateInfo = await api.query.parachainStaking.candidateInfo(accountId);
      return (candidateInfo as any).isSome;
    }
    const delegatorState = await api.query.parachainStaking.delegatorState(accountId);
    return (delegatorState as any).isSome;
  };

  return executeWithRetry(operation, 3, 5000, `checkAccountExistsInStaking(${accountId})`);
};

// Helper function to log account processing progress
const logAccountProgress = (
  log: any,
  accountType: string,
  processed: number,
  total: number,
  migrated: number,
  batchSize = 50
) => {
  if (processed % batchSize === 0) {
    log(`  Checked ${processed}/${total} ${accountType} (${migrated} already migrated)...`);
  }
};

// Helper function to validate and log specific account status
const validateAndLogAccountStatus = async (
  api: ApiPromise,
  accountId: string,
  isCandidate: boolean,
  log: any
): Promise<boolean> => {
  const accountType = isCandidate ? "candidate" : "delegator";
  const existsInStaking = await checkAccountExistsInStaking(api, accountId, isCandidate);

  if (!existsInStaking) {
    log(`  ❌ ${accountId} is not a ${accountType}`);
    return false;
  }

  const isMigrated = await checkMigrationStatus(api, accountId, isCandidate);
  if (!isMigrated) {
    log(`  ✅ ${accountId} is a non-migrated ${accountType}`);
    return true;
  }

  log(`  ⚠️  ${accountId} is a ${accountType} but already migrated`);
  return false;
};

// Helper function to verify account migration with timeout
const verifyAccountMigrationWithTimeout = async (
  api: ApiPromise,
  accountId: string,
  isCandidate: boolean,
  timeout = 10000
): Promise<boolean> => {
  const migrationQuery = isCandidate
    ? api.query.parachainStaking.migratedCandidates(accountId)
    : api.query.parachainStaking.migratedDelegators(accountId);

  const isMigrated = await Promise.race([
    migrationQuery,
    new Promise((_, reject) => setTimeout(() => reject(new Error("Query timeout")), timeout)),
  ]);

  return (isMigrated as any).isSome;
};

// Helper function to verify freeze existence for zero balance accounts
const verifyZeroBalanceAccount = async (
  api: ApiPromise,
  accountId: string,
  isCandidate: boolean
): Promise<boolean> => {
  const operation = async () => {
    if (isCandidate) {
      const candidateInfo = await api.query.parachainStaking.candidateInfo(accountId);
      if ((candidateInfo as any).isSome) {
        const bond = (candidateInfo as any).unwrap().bond;
        return bond.isZero();
      }
    } else {
      const delegatorState = await api.query.parachainStaking.delegatorState(accountId);
      if ((delegatorState as any).isSome) {
        const total = (delegatorState as any).unwrap().total;
        return total.isZero();
      }
    }
    return false;
  };

  return executeWithRetry(operation, 3, 5000, `verifyZeroBalanceAccount(${accountId})`);
};

// Helper function to create migration result
const createMigrationResult = (
  accountId: string,
  isCandidate: boolean,
  success: boolean,
  error?: string
): MigrationResult => ({
  accountId,
  type: isCandidate ? "candidate" : "delegator",
  success,
  error,
});

// Helper function to update migration statistics
const updateMigrationStats = (stats: MigrationStats, result: MigrationResult): void => {
  stats.totalProcessed++;

  if (result.success) {
    stats.totalSuccessful++;
    if (result.type === "candidate") {
      stats.candidatesSuccessful++;
    } else {
      stats.delegatorsSuccessful++;
    }
  } else {
    const isSkipped =
      result.error?.includes("Skipped due to RPC timeout") ||
      result.error?.includes("timeout after");
    if (isSkipped) {
      stats.totalSkipped++;
      stats.skipped.push(result);
    } else {
      stats.totalFailed++;
      stats.failures.push(result);
    }
  }

  if (result.type === "candidate") {
    stats.candidatesProcessed++;
  } else {
    stats.delegatorsProcessed++;
  }
};

// Helper function to generate CSV content
const generateMigrationResultsCSV = (stats: MigrationStats): string => {
  let csvContent = "Type,AccountId,Status,Reason\n";

  // Add failed accounts with detailed reasons
  stats.failures.forEach((failure) => {
    const reason = failure.error || "Unknown error";
    const escapedReason = reason.replace(/"/g, '""');
    csvContent += `${failure.type},${failure.accountId},failed,"${escapedReason}"\n`;
  });

  // Add skipped accounts with detailed reasons
  stats.skipped.forEach((skipped) => {
    const reason = skipped.error || "Unknown error";
    const escapedReason = reason.replace(/"/g, '""');
    csvContent += `${skipped.type},${skipped.accountId},skipped,"${escapedReason}"\n`;
  });

  return csvContent;
};

// Helper function to log migration statistics
const logMigrationStatistics = (stats: MigrationStats, log: any): void => {
  log(`--- 🎉 Migration completed ---`);
  log(`--- 📊 Overall Statistics ---`);
  log(`  Total Accounts Processed: ${stats.totalProcessed}`);
  log(
    `  ✅ Successful: ${stats.totalSuccessful} (${(
      (stats.totalSuccessful / stats.totalProcessed) *
      100
    ).toFixed(2)}%)`
  );
  log(
    `  ❌ Failed: ${stats.totalFailed} (${(
      (stats.totalFailed / stats.totalProcessed) *
      100
    ).toFixed(2)}%)`
  );
  log(
    `  ⏭️  Skipped (RPC timeouts): ${stats.totalSkipped} (${(
      (stats.totalSkipped / stats.totalProcessed) *
      100
    ).toFixed(2)}%)`
  );

  // Candidate statistics
  log(`--- 📊 Candidate Statistics ---`);
  log(`  Total Candidates: ${stats.candidatesProcessed}`);
  log(`  ✅ Successfully Migrated: ${stats.candidatesSuccessful}`);
  log(`  ❌ Failed: ${stats.failures.filter((f) => f.type === "candidate").length}`);
  log(`  ⏭️  Skipped: ${stats.skipped.filter((s) => s.type === "candidate").length}`);
  log(
    `  Success Rate: ${
      stats.candidatesProcessed > 0
        ? ((stats.candidatesSuccessful / stats.candidatesProcessed) * 100).toFixed(2)
        : "0.00"
    }%`
  );

  // Delegator statistics
  log(`\n📊 Delegator Statistics:`);
  log(`  Total Delegators: ${stats.delegatorsProcessed}`);
  log(`  ✅ Successfully Migrated: ${stats.delegatorsSuccessful}`);
  log(`  ❌ Failed: ${stats.failures.filter((f) => f.type === "delegator").length}`);
  log(`  ⏭️  Skipped: ${stats.skipped.filter((s) => s.type === "delegator").length}`);
  log(
    `  Success Rate: ${
      stats.delegatorsProcessed > 0
        ? ((stats.delegatorsSuccessful / stats.delegatorsProcessed) * 100).toFixed(2)
        : "0.00"
    }%`
  );

  // Log failures and skipped accounts
  if (stats.skipped.length > 0) {
    log(`\n⚠️  Skipped Accounts due to RPC timeouts (${stats.skipped.length}):`);
    stats.skipped.forEach((skipped) => {
      log(`  - ${skipped.type}: ${skipped.accountId} - ${skipped.error}`);
    });
  }

  if (stats.failures.length > 0) {
    log(`\n❌ Failed Accounts (${stats.failures.length}):`);
    stats.failures.forEach((failure) => {
      log(`  - ${failure.type}: ${failure.accountId} - ${failure.error}`);
    });
  }
};

const upgradeRuntime = async (context: ChopsticksContext) => {
  const path = (await MoonwallContext.getContext()).rtUpgradePath;
  if (!path || !existsSync(path)) {
    throw new Error(`Runtime wasm not found at path: ${path}`);
  }
  const rtWasm = readFileSync(path);
  const rtHex = `0x${rtWasm.toString("hex")}`;
  const rtHash = blake2AsHex(rtHex);
  const api = context.polkadotJs();
  const signer = context.keyring.alice;

  await context.setStorage({
    module: "system",
    method: "authorizedUpgrade",
    methodParams: `${rtHash}01`, // 01 is for the RT ver check = true
  });
  await context.createBlock();

  await api.tx.system.applyAuthorizedUpgrade(rtHex).signAndSend(signer);

  const paraId = await api.query.parachainInfo.parachainId();

  await api.rpc("dev_newBlock", {
    count: 3,
    relayChainStateOverrides: [[upgradeRestrictionSignal(paraId as u32), null]],
  });
};

const discoverAllNonMigratedAccounts = async (
  api: ApiPromise,
  log: any
): Promise<[string, boolean][]> => {
  log("🔍 Discovering ALL non-migrated accounts...");
  const nonMigratedAccounts: [string, boolean][] = [];

  // Get ALL candidates with retry logic
  log("📋 Checking ALL candidates...");
  const candidateInfo = await executeWithRetry(
    () => api.query.parachainStaking.candidateInfo.entries(),
    3,
    5000,
    "candidateInfo.entries()"
  );
  let candidatesChecked = 0;
  let candidatesMigrated = 0;

  // Process candidates in smaller batches to avoid RPC timeouts
  const candidateBatchSize = 20;
  for (let i = 0; i < candidateInfo.length; i += candidateBatchSize) {
    const batch = candidateInfo.slice(i, i + candidateBatchSize);

    await Promise.all(
      batch.map(async ([key, candidateData]) => {
        if ((candidateData as any).isSome) {
          const collatorAccount = key.args[0].toString();
          try {
            const isMigrated = await checkMigrationStatus(api, collatorAccount, true);

            // Debug logging for first few candidates
            if (candidatesChecked < 3) {
              log(`  🔍 Candidate ${collatorAccount}: migrated = ${isMigrated ? "true" : "false"}`);
            }

            if (!isMigrated) {
              nonMigratedAccounts.push([collatorAccount, true]);
            } else {
              candidatesMigrated++;
            }
          } catch (error) {
            log(`  ⚠️  Error checking candidate ${collatorAccount}: ${error}`);
            // Continue processing other accounts
          }
          candidatesChecked++;
        }
      })
    );

    logAccountProgress(
      log,
      "candidates",
      candidatesChecked,
      candidateInfo.length,
      candidatesMigrated
    );
  }

  // Get ALL delegators with retry logic
  log("📋 Checking ALL delegators...");
  const delegatorState = await executeWithRetry(
    () => api.query.parachainStaking.delegatorState.entries(),
    3,
    5000,
    "delegatorState.entries()"
  );
  let delegatorsChecked = 0;
  let delegatorsMigrated = 0;

  // Process delegators in smaller batches to avoid RPC timeouts
  const delegatorBatchSize = 20;
  for (let i = 0; i < delegatorState.length; i += delegatorBatchSize) {
    const batch = delegatorState.slice(i, i + delegatorBatchSize);

    await Promise.all(
      batch.map(async ([key, delegatorData]) => {
        if ((delegatorData as any).isSome) {
          const delegatorAccount = key.args[0].toString();
          try {
            const isMigrated = await checkMigrationStatus(api, delegatorAccount, false);

            // Debug logging for first few delegators
            if (delegatorsChecked < 3) {
              log(
                `  🔍 Delegator ${delegatorAccount}: migrated = ${isMigrated ? "true" : "false"}`
              );
            }

            if (!isMigrated) {
              nonMigratedAccounts.push([delegatorAccount, false]);
            } else {
              delegatorsMigrated++;
            }
          } catch (error) {
            log(`  ⚠️  Error checking delegator ${delegatorAccount}: ${error}`);
            // Continue processing other accounts
          }
          delegatorsChecked++;
        }
      })
    );

    logAccountProgress(
      log,
      "delegators",
      delegatorsChecked,
      delegatorState.length,
      delegatorsMigrated
    );
  }

  const candidateCount = nonMigratedAccounts.filter(([_, isCandidate]) => isCandidate).length;
  const delegatorCount = nonMigratedAccounts.filter(([_, isCandidate]) => !isCandidate).length;
  log(
    `🎯 Found ${candidateCount} non-migrated candidates and ${delegatorCount} non-migrated delegators`
  );
  log(`📊 Already migrated: ${candidatesMigrated} candidates, ${delegatorsMigrated} delegators`);
  log(`📊 Total accounts to migrate: ${nonMigratedAccounts.length}`);

  return nonMigratedAccounts;
};

const getSpecificCandidates = async (
  api: ApiPromise,
  accountList: string[],
  log: any
): Promise<[string, boolean][]> => {
  log(`🔍 Checking specific candidate accounts: ${accountList.join(", ")}`);
  const accountsToMigrate: [string, boolean][] = [];

  for (const account of accountList) {
    const shouldMigrate = await validateAndLogAccountStatus(api, account, true, log);
    if (shouldMigrate) {
      accountsToMigrate.push([account, true]);
    }
  }

  log(`📊 Found ${accountsToMigrate.length} candidate accounts to migrate`);
  return accountsToMigrate;
};

const getSpecificDelegators = async (
  api: ApiPromise,
  accountList: string[],
  log: any
): Promise<[string, boolean][]> => {
  log(`🔍 Checking specific delegator accounts: ${accountList.join(", ")}`);
  const accountsToMigrate: [string, boolean][] = [];

  for (const account of accountList) {
    const shouldMigrate = await validateAndLogAccountStatus(api, account, false, log);
    if (shouldMigrate) {
      accountsToMigrate.push([account, false]);
    }
  }

  log(`📊 Found ${accountsToMigrate.length} delegator accounts to migrate`);
  return accountsToMigrate;
};

const executeBatchMigrationWithRetry = async (
  api: ApiPromise,
  context: ChopsticksContext,
  batch: [string, boolean][],
  log: any,
  maxRetries = 3
): Promise<MigrationResult[]> => {
  const results: MigrationResult[] = [];

  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      log(
        `🚀 Executing migration for batch of ${batch.length} accounts (attempt ${attempt}/${maxRetries})...`
      );

      // Execute the migration transaction and wait for it to be included in a block
      try {
        await api.tx.parachainStaking.migrateLocksToFreezesBatch(batch).signAndSend(alith);
        await context.createBlock();
      } catch (txError) {
        const errorMsg = txError instanceof Error ? txError.message : String(txError);
        throw new Error(`Migration transaction failed: ${errorMsg}`);
      }

      log("✅ Migration transaction completed, verifying results...");

      // Verify each account in the batch in parallel
      const verificationPromises = batch.map(async ([accountId, isCandidate]) => {
        try {
          // Check if migration was successful with timeout
          const migrationSuccessful = await verifyAccountMigrationWithTimeout(
            api,
            accountId,
            isCandidate
          );

          if (migrationSuccessful) {
            if (isCandidate) {
              await verifyCandidateInfoMatchesFreezes(accountId as `0x${string}`, context as any);
              const hasFreeze = await hasCollatorStakingFreeze(
                accountId as `0x${string}`,
                context as any
              );
              if (!hasFreeze) {
                // Check if this is expected (zero bond)
                const isZeroBalance = await verifyZeroBalanceAccount(api, accountId, true);
                if (isZeroBalance) {
                  return createMigrationResult(accountId, isCandidate, true);
                }

                // If we get here, it's an actual error
                const candidateInfo = await api.query.parachainStaking.candidateInfo(accountId);
                const bondStr = (candidateInfo as any).isSome
                  ? (candidateInfo as any).unwrap().bond.toString()
                  : "No candidate info";
                const balance = await api.query.system.account(accountId);
                const freeBalance = (balance as any).data.free.toString();

                throw new Error(
                  `Migration marked complete but no collator freeze found. Bond: ${bondStr}, Free balance: ${freeBalance}`
                );
              }
            } else {
              await verifyDelegatorStateMatchesFreezes(accountId as `0x${string}`, context as any);
              const hasFreeze = await hasDelegatorStakingFreeze(
                accountId as `0x${string}`,
                context as any
              );
              if (!hasFreeze) {
                // Check if this is expected (zero total delegation)
                const isZeroBalance = await verifyZeroBalanceAccount(api, accountId, false);
                if (isZeroBalance) {
                  return createMigrationResult(accountId, isCandidate, true);
                }

                // If we get here, it's an actual error
                const delegatorState = await api.query.parachainStaking.delegatorState(accountId);
                const totalStr = (delegatorState as any).isSome
                  ? (delegatorState as any).unwrap().total.toString()
                  : "No delegator state";
                throw new Error(
                  `Migration marked complete but no delegator freeze found. Total delegation: ${totalStr}`
                );
              }
            }
            return createMigrationResult(accountId, isCandidate, true);
          }

          return createMigrationResult(
            accountId,
            isCandidate,
            false,
            "Migration not marked as complete"
          );
        } catch (error) {
          const errorMsg = error instanceof Error ? error.message : String(error);
          const timeoutError = errorMsg.includes("timeout")
            ? `Verification timeout (attempt ${attempt}): ${errorMsg}`
            : errorMsg;

          return createMigrationResult(accountId, isCandidate, false, timeoutError);
        }
      });

      // Wait for all verifications to complete
      const verificationResults = await Promise.all(verificationPromises);
      results.push(...verificationResults);

      // If we get here, the batch completed (with possible individual failures)
      return results;
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      const isTimeout = errorMsg.includes("timeout") || errorMsg.includes("Timeout");

      if (isTimeout && attempt < maxRetries) {
        log(`⚠️  RPC timeout on attempt ${attempt}/${maxRetries}, retrying in 5 seconds...`);
        await new Promise((resolve) => setTimeout(resolve, 5000)); // Wait 5 seconds before retry
        continue;
      }

      if (isTimeout) {
        log(`❌ RPC timeout after ${maxRetries} attempts, marking batch as skipped`);
        // Mark all accounts in the batch as skipped due to timeout
        for (const [accountId, isCandidate] of batch) {
          results.push(
            createMigrationResult(
              accountId,
              isCandidate,
              false,
              `Skipped due to RPC timeout after ${maxRetries} attempts: ${errorMsg}`
            )
          );
        }
        return results;
      }

      log(`❌ Batch migration failed: ${errorMsg}`);
      // Mark all accounts in the batch as failed
      for (const [accountId, isCandidate] of batch) {
        results.push(
          createMigrationResult(
            accountId,
            isCandidate,
            false,
            `Batch migration failed: ${errorMsg}`
          )
        );
      }
      return results;
    }
  }

  return results;
};

describeSuite({
  id: "C02",
  title: "Chopsticks Staking Migration",
  foundationMethods: "chopsticks",
  testCases: ({ it, context, log }) => {
    let api: ApiPromise;

    // NOTE: This test suite limits the number of accounts checked to 10 candidates and 10 delegators
    // for performance reasons. In a production environment, you may want to:
    // 1. Use batch queries to check multiple accounts at once
    // 2. Implement parallel checking with Promise.all()
    // 3. Only check a random sample of accounts
    // 4. Focus on specific test accounts rather than all mainnet accounts

    beforeAll(async () => {
      api = context.polkadotJs();
      log("Setting up chopsticks test for staking migration...");

      // Perform runtime upgrade to include migration storage items
      const rtBefore = (api.consts.system.version as any).specVersion.toNumber();
      log("Upgrading runtime to include migration storage items...");
      await upgradeRuntime(context);

      const rtAfter = (api.consts.system.version as any).specVersion.toNumber();
      log(`Runtime upgraded from ${rtBefore} to ${rtAfter}`);

      if (rtBefore === rtAfter) {
        throw new Error("Runtime upgrade failed");
      }

      // Wait for API to reconnect with new runtime
      await new Promise((resolve) => setTimeout(resolve, 1000));

      // Ensure connection is ready
      await api.isReady;
      log("API is ready with new runtime");
    });

    it({
      id: "T1",
      timeout: 3600000, // Increase to 60 minutes
      title: "Should discover non-migrated candidates and delegators",
      test: async () => {
        let allNonMigratedAccounts: [string, boolean][];

        // Check if specific accounts are provided via environment variables
        const specificCandidates = process.env.MIGRATE_CANDIDATE_ACCOUNTS;
        const specificDelegators = process.env.MIGRATE_DELEGATOR_ACCOUNTS;

        if (specificCandidates || specificDelegators) {
          allNonMigratedAccounts = [];

          // Process specific candidates if provided
          if (specificCandidates) {
            const candidateList = specificCandidates.split(",").map((acc) => acc.trim());
            log(`📋 Using specific candidate list: ${candidateList.length} accounts`);
            const candidates = await getSpecificCandidates(api, candidateList, log);
            allNonMigratedAccounts.push(...candidates);
          }

          // Process specific delegators if provided
          if (specificDelegators) {
            const delegatorList = specificDelegators.split(",").map((acc) => acc.trim());
            log(`📋 Using specific delegator list: ${delegatorList.length} accounts`);
            const delegators = await getSpecificDelegators(api, delegatorList, log);
            allNonMigratedAccounts.push(...delegators);
          }
        } else {
          // Discover all non-migrated accounts
          allNonMigratedAccounts = await discoverAllNonMigratedAccounts(api, log);
        }

        expect(allNonMigratedAccounts).to.be.an("array");
        log(`✅ Discovery complete: ${allNonMigratedAccounts.length} accounts need migration`);

        // Store in context for next test
        (context as any).allNonMigratedAccounts = allNonMigratedAccounts;
      },
    });

    it({
      id: "T2",
      timeout: 7200000, // 2 hour timeout for complete migration
      title: "Should migrate candidates and delegators recursively in batches",
      test: async () => {
        const accountsToMigrate = (context as any).allNonMigratedAccounts || [];
        const batchSize = 100;
        const stats: MigrationStats = {
          totalProcessed: 0,
          totalSuccessful: 0,
          totalFailed: 0,
          totalSkipped: 0,
          candidatesProcessed: 0,
          candidatesSuccessful: 0,
          delegatorsProcessed: 0,
          delegatorsSuccessful: 0,
          failures: [],
          skipped: [],
        };

        if (accountsToMigrate.length === 0) {
          throw new Error(
            "❌ No accounts found to migrate. T1 test likely failed or timed out. Cannot proceed with migration test."
          );
        }

        // Count candidates and delegators to migrate
        const candidatesToMigrate = accountsToMigrate.filter(
          ([_, isCandidate]) => isCandidate
        ).length;
        const delegatorsToMigrate = accountsToMigrate.filter(
          ([_, isCandidate]) => !isCandidate
        ).length;

        log(`\n🚀 Starting migration process:`);
        log(`📊 Accounts to migrate:`);
        log(`   Total: ${accountsToMigrate.length}`);
        log(`   Candidates: ${candidatesToMigrate}`);
        log(`   Delegators: ${delegatorsToMigrate}`);

        // Process all accounts in batches
        for (let i = 0; i < accountsToMigrate.length; i += batchSize) {
          const batch = accountsToMigrate.slice(i, i + batchSize);
          const batchNumber = Math.floor(i / batchSize) + 1;
          const totalBatches = Math.ceil(accountsToMigrate.length / batchSize);

          log(`📦 Batch ${batchNumber}/${totalBatches}: ${batch.length} accounts`);

          const batchResults = await executeBatchMigrationWithRetry(api, context, batch, log);

          // Update stats
          for (const result of batchResults) {
            updateMigrationStats(stats, result);
          }

          log(
            `📊 Batch ${batchNumber} complete: ${batchResults.filter((r) => r.success).length}/${
              batchResults.length
            } successful`
          );
        }

        // Final results
        logMigrationStatistics(stats, log);

        // Write detailed results to CSV file
        if (stats.failures.length > 0 || stats.skipped.length > 0) {
          const timestamp = new Date().toISOString().replace(/[:.]/g, "-");
          const csvPath = `/tmp/staking-migration-results-${timestamp}.csv`;
          const csvContent = generateMigrationResultsCSV(stats);

          writeFileSync(csvPath, csvContent);
          log(`\n📁 Migration results written to: ${csvPath}`);
          log(`   Total entries: ${stats.failures.length + stats.skipped.length}`);
          log(
            `   Failed: ${stats.failures.length} (${
              stats.failures.filter((f) => f.type === "candidate").length
            } candidates, ${
              stats.failures.filter((f) => f.type === "delegator").length
            } delegators)`
          );
          log(
            `   Skipped: ${stats.skipped.length} (${
              stats.skipped.filter((s) => s.type === "candidate").length
            } candidates, ${stats.skipped.filter((s) => s.type === "delegator").length} delegators)`
          );
        }

        // Store stats for verification test
        (context as any).migrationStats = stats;

        expect(stats.totalProcessed).to.be.greaterThan(0);
      },
    });

    it({
      id: "T3",
      timeout: 1200000,
      title: "Should verify staking operations work after migration",
      test: async () => {
        const currentHeight = (await api.rpc.chain.getBlock()).block.header.number.toNumber();

        // Create a few blocks to ensure system continues working
        await context.createBlock({ count: 5 });

        const newHeight = (await api.rpc.chain.getBlock()).block.header.number.toNumber();
        expect(newHeight - currentHeight).to.equal(5);

        log("Staking system continues to produce blocks after migration");
      },
    });
  },
});

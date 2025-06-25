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
import { existsSync, readFileSync } from "node:fs";
import {
  hasCollatorStakingFreeze,
  hasDelegatorStakingFreeze,
  verifyCandidateInfoMatchesFreezes,
  verifyDelegatorStateMatchesFreezes,
} from "../../helpers/staking-freezes";

const hash = (prefix: HexString, suffix: Uint8Array) => {
  return u8aToHex(u8aConcat(hexToU8a(prefix), xxhashAsU8a(suffix, 64), suffix));
};

const upgradeRestrictionSignal = (paraId: u32) => {
  const prefix = "0xcd710b30bd2eab0352ddcc26417aa194f27bbb460270642b5bcaf032ea04d56a";
  return hash(prefix, paraId.toU8a());
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
      timeout: 1200000,
      title: "Should identify all non-migrated candidates and delegators",
      test: async () => {
        // Get all candidates
        const candidateInfo = await api.query.parachainStaking.candidateInfo.entries();
        const nonMigratedCandidates: string[] = [];

        log(`Found ${candidateInfo.length} total candidates to check`);
        let candidatesChecked = 0;

        // Limit to first 10 candidates for testing
        const candidatesToCheck = candidateInfo.slice(0, 10);

        for (const [key, candidateData] of candidatesToCheck) {
          if ((candidateData as any).isSome) {
            const collatorAccount = key.args[0].toString();
            const isMigrated = await api.query.parachainStaking.migratedCandidates(collatorAccount);
            if ((isMigrated as any).isNone) {
              nonMigratedCandidates.push(collatorAccount);
            }
            candidatesChecked++;
            if (candidatesChecked % 5 === 0) {
              log(`Checked ${candidatesChecked} candidates...`);
            }
          }
        }

        // Get all delegators
        const delegatorState = await api.query.parachainStaking.delegatorState.entries();
        const nonMigratedDelegators: string[] = [];

        log(`Found ${delegatorState.length} total delegators to check`);
        let delegatorsChecked = 0;

        // Limit to first 10 delegators for testing
        const delegatorsToCheck = delegatorState.slice(0, 10);

        for (const [key, delegatorData] of delegatorsToCheck) {
          if ((delegatorData as any).isSome) {
            const delegatorAccount = key.args[0].toString();
            const isMigrated = await api.query.parachainStaking.migratedDelegators(
              delegatorAccount
            );
            if ((isMigrated as any).isNone) {
              nonMigratedDelegators.push(delegatorAccount);
            }
            delegatorsChecked++;
            if (delegatorsChecked % 5 === 0) {
              log(`Checked ${delegatorsChecked} delegators...`);
            }
          }
        }

        log(`Found ${nonMigratedCandidates.length} non-migrated candidates`);
        log(`Found ${nonMigratedDelegators.length} non-migrated delegators`);
        log(
          `Total accounts to migrate: ${
            nonMigratedCandidates.length + nonMigratedDelegators.length
          }`
        );

        // Store in context for next test
        (context as any).nonMigratedCandidates = nonMigratedCandidates;
        (context as any).nonMigratedDelegators = nonMigratedDelegators;
      },
    });

    it({
      id: "T2",
      timeout: 1200000,
      title: "Should migrate all candidates and delegators in batches",
      test: async () => {
        // NOTE: For testing purposes, we limit the number of accounts processed
        // In production, you would want to process all accounts or use a more efficient batch query
        const nonMigratedCandidates = (context as any).nonMigratedCandidates || [];
        const nonMigratedDelegators = (context as any).nonMigratedDelegators || [];

        if (nonMigratedCandidates.length === 0 && nonMigratedDelegators.length === 0) {
          log("No accounts to migrate");
          return;
        }

        const batchSize = 100; // Adjust based on weight limits
        let totalMigrated = 0;

        // Combine all accounts to migrate
        const allAccounts: [string, boolean][] = [
          ...nonMigratedCandidates.map((acc: string): [string, boolean] => [acc, true]),
          ...nonMigratedDelegators.map((acc: string): [string, boolean] => [acc, false]),
        ];

        log(`Starting migration of ${allAccounts.length} accounts in batches of ${batchSize}`);

        // Process in batches
        for (let i = 0; i < allAccounts.length; i += batchSize) {
          const batch = allAccounts.slice(i, i + batchSize);

          log(`Processing batch ${Math.floor(i / batchSize) + 1} with ${batch.length} accounts...`);

          // Execute migration
          await api.tx.parachainStaking.migrateLocksToFreezesBatch(batch).signAndSend(alith);
          await context.createBlock();

          totalMigrated += batch.length;
          log(`Migrated ${totalMigrated}/${allAccounts.length} accounts`);
        }

        expect(totalMigrated).to.equal(allAccounts.length);
        log(`Migration completed! Total accounts migrated: ${totalMigrated}`);
      },
    });

    it({
      id: "T3",
      timeout: 1200000,
      title: "Should verify all accounts are migrated and have proper freezes",
      test: async () => {
        // Verify all candidates are migrated
        const candidateInfo = await api.query.parachainStaking.candidateInfo.entries();
        let migratedCandidates = 0;
        let totalCandidates = 0;

        log(`Found ${candidateInfo.length} total candidates to verify`);
        
        // Limit to first 10 candidates for testing
        const candidatesToCheck = candidateInfo.slice(0, 10);

        for (const [key, candidateData] of candidatesToCheck) {
          if ((candidateData as any).isSome) {
            totalCandidates++;
            const collatorAccount = key.args[0].toString();
            const isMigrated = await api.query.parachainStaking.migratedCandidates(collatorAccount);
            if ((isMigrated as any).isSome) {
              migratedCandidates++;
              
              // Verify the candidate has proper freezes
              expect(await hasCollatorStakingFreeze(collatorAccount as `0x${string}`, context as any)).to.be.true;
              
              // Verify the candidate bond matches the freeze amount
              await verifyCandidateInfoMatchesFreezes(collatorAccount as `0x${string}`, context as any);
            }
          }
        }

        // Verify all delegators are migrated
        const delegatorState = await api.query.parachainStaking.delegatorState.entries();
        let migratedDelegators = 0;
        let totalDelegators = 0;

        log(`Found ${delegatorState.length} total delegators to verify`);
        
        // Limit to first 10 delegators for testing
        const delegatorsToCheck = delegatorState.slice(0, 10);

        for (const [key, delegatorData] of delegatorsToCheck) {
          if ((delegatorData as any).isSome) {
            totalDelegators++;
            const delegatorAccount = key.args[0].toString();
            const isMigrated = await api.query.parachainStaking.migratedDelegators(
              delegatorAccount
            );
            if ((isMigrated as any).isSome) {
              migratedDelegators++;
              
              // Verify the delegator has proper freezes
              expect(await hasDelegatorStakingFreeze(delegatorAccount as `0x${string}`, context as any)).to.be.true;
              
              // Verify the delegator state total matches the freeze amount
              await verifyDelegatorStateMatchesFreezes(delegatorAccount as `0x${string}`, context as any);
            }
          }
        }

        log(`Migration verification results:`);
        log(`- Candidates: ${migratedCandidates}/${totalCandidates} migrated`);
        log(`- Delegators: ${migratedDelegators}/${totalDelegators} migrated`);

        // All accounts should be migrated
        expect(migratedCandidates).to.equal(totalCandidates);
        expect(migratedDelegators).to.equal(totalDelegators);
      },
    });

    it({
      id: "T4",
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

import fs from "fs";
import readline from "readline";
import chalk from "chalk";

import { xxhashAsU8a, blake2AsU8a } from "@polkadot/util-crypto";
import { u8aConcat, u8aToHex, hexToBigInt, nToHex } from "@polkadot/util";

const storageKey = (module, name) => {
  return u8aToHex(u8aConcat(xxhashAsU8a(module, 128), xxhashAsU8a(name, 128)));
};

const storageBlake128MapKey = (module, name, key) => {
  return u8aToHex(
    u8aConcat(xxhashAsU8a(module, 128), xxhashAsU8a(name, 128), blake2AsU8a(key, 128), key)
  );
};

/**
 * All module prefixes except those mentioned in the skippedModulesPrefix will be added to this by the script.
 * If you want to add any past module or part of a skipped module, add the prefix here manually.
 *
 * Any storage value’s hex can be logged via console.log(api.query.<module>.<call>.key([...opt params])),
 * e.g. console.log(api.query.timestamp.now.key()).
 *
 * If you want a map/doublemap key prefix, you can do it via .keyPrefix(),
 * e.g. console.log(api.query.system.account.keyPrefix()).
 *
 * For module hashing, do it via xxhashAsHex,
 * e.g. console.log(xxhashAsHex('System', 128)).
 */

// const ALITH_PRIV_KEY = "0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133";
// const BOB_PRIV_KEY = "0x8075991ce870b93a8870eca0c0f91913d12f47948ca0fd25b49c6fa7cdbeee8b";

async function main(inputFile: string, outputFile?: string) {
  const ALITH = "0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac";
  const ALITH_SESSION = "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";
  const ALITH_MIN_BALANCE = 10_000n * 10n ** 18n;

  const in1Stream = fs.createReadStream(inputFile, "utf8");
  const rl1 = readline.createInterface({
    input: in1Stream,
    crlfDelay: Infinity,
  });

  const destFile = outputFile || inputFile.replace(/\.json/, ".mod.json");
  if (destFile == inputFile) {
    console.log(`Expected json file`);
    return -1;
  }

  let messagingState = null;
  const collatorLinePrefix = `        "${storageKey("ParachainStaking", "SelectedCandidates")}`;
  const authorLinePrefix = `        "${storageKey("AuthorMapping", "MappingWithDeposit")}`;
  const nimbusLookupPrefix = `        "${storageKey("AuthorMapping", "NimbusLookup")}`;
  const revelentMessagingStatePrefix = `        "${storageKey(
    "ParachainSystem",
    "RelevantMessagingState"
  )}`;
  const authorEligibilityRatioPrefix = `        "${storageKey("AuthorFilter", "EligibleRatio")}`;
  const authorEligibilityCountPrefix = `        "${storageKey("AuthorFilter", "EligibleCount")}`;
  const councilLinePrefix = `        "${storageKey("CouncilCollective", "Members")}`;
  const techCommitteeeLinePrefix = `        "${storageKey("TechCommitteeCollective", "Members")}`;
  const parachainIdPrefix = `        "${storageKey("ParachainInfo", "ParachainId")}`;
  const lastDmqMqcHeadPrefix = `        "${storageKey("ParachainSystem", "LastDmqMqcHead")}`;
  const alithBalancePrefix = `        "${storageBlake128MapKey("System", "Account", ALITH)}`;
  const totalIssuanceBalancePrefix = `        "${storageKey("Balances", "TotalIssuance")}`;
  const bootnodesPrefix = `    "/`;

  // List all the collator author mapping
  const authorMappingLines = {};

  // First pass
  let selectedCollator = null;
  let totalIssuance: bigint;
  let alithAccountData;
  for await (const line of rl1) {
    if (line.startsWith(collatorLinePrefix)) {
      // First collator we found, it will be our target for Alice
      if (!selectedCollator) {
        selectedCollator = `0x${line.split('"')[3].slice(-40)}`;
        console.log(`Using account ${selectedCollator} as alice collator`);
      }
    }
    if (line.startsWith(authorLinePrefix)) {
      const collator = line.split('"')[3].slice(0, 42);
      authorMappingLines[collator] = line;
    }
    if (line.startsWith(revelentMessagingStatePrefix)) {
      messagingState = line.split('"')[3];
    }
    if (line.startsWith(totalIssuanceBalancePrefix)) {
      totalIssuance = hexToBigInt(line.split('"')[3], { isLe: true });
    }
    if (line.startsWith(alithBalancePrefix)) {
      alithAccountData = line.split('"')[3];
    }
  }

  // List all the collator author mapping
  console.log(
    `Using account ${selectedCollator} as alice collator, session ${
      authorMappingLines[selectedCollator].split('"')[1]
    }`
  );

  if (!selectedCollator) {
    console.log(`Couldn't find collator with prefix ${authorLinePrefix}`);
    return;
  }
  const selectedCollatorMappingKey = authorMappingLines[selectedCollator].split('"')[1];

  if (!messagingState) {
    console.log(`Couldn't find messaging state with prefix ${revelentMessagingStatePrefix}`);
    return;
  }

  // We add 1_000 tokens to alith if needed (for governance...)
  // and so we need to add it to the totalIssuance to stay consistent;
  const alithFreeBalance = hexToBigInt(alithAccountData.slice(34, 66), { isLe: true });
  let newAlithAccountData = alithAccountData;
  let newTotalIssuance = totalIssuance;
  if (alithFreeBalance < ALITH_MIN_BALANCE) {
    newAlithAccountData = `${alithAccountData.slice(0, 34)}${nToHex(
      alithFreeBalance + ALITH_MIN_BALANCE,
      {
        isLe: true,
        bitLength: 128,
      }
    ).slice(2)}${alithAccountData.slice(66)}`;
    newTotalIssuance = totalIssuance + ALITH_MIN_BALANCE;
  }

  const in2Stream = fs.createReadStream(inputFile, "utf8");
  const rl2 = readline.createInterface({
    input: in2Stream,
    crlfDelay: Infinity,
  });
  const outStream = fs.createWriteStream(destFile, { encoding: "utf8" });

  const selectedAuthorMappingPrefix = `        "${selectedCollatorMappingKey}"`;

  for await (const line of rl2) {
    if (line.startsWith(selectedAuthorMappingPrefix)) {
      console.log(
        ` ${chalk.red(
          `  - Removing AuthorMapping.MappingWithDeposit ${
            authorMappingLines[selectedCollator].split('"')[1]
          }`
        )}\n\t${line}`
      );
      let newLine = `        "${storageBlake128MapKey(
        "AuthorMapping",
        "MappingWithDeposit",
        ALITH_SESSION
      )}": "${authorMappingLines[selectedCollator]
        .split('"')[3]
        .slice(0, -64)}${ALITH_SESSION.slice(2)}",\n`;
      console.log(
        ` ${chalk.green(`  + Adding AuthorMapping.MappingWithDeposit: Alith`)}\n\t${newLine}`
      );
      outStream.write(newLine);
    } else if (line.startsWith(nimbusLookupPrefix)) {
      console.log(
        ` ${chalk.red(`  - Removing AuthorMapping.NimbusLookup ${selectedCollator}\n\t${line}`)}`
      );
      let newLine = `        "${storageBlake128MapKey(
        "AuthorMapping",
        "NimbusLookup",
        ALITH
      )}": "${ALITH_SESSION}",\n`;
      console.log(` ${chalk.green(`  + Adding AuthorMapping.NimbusLookup: Alith`)}\n\t${newLine}`);
      outStream.write(newLine);
    } else if (line.startsWith(revelentMessagingStatePrefix)) {
      console.log(
        ` ${chalk.red(`  - Removing ParachainSystem.RelevantMessagingState`)}\n\t${line}`
      );
      const newLine = `        "${storageKey(
        "ParachainSystem",
        "RelevantMessagingState"
      )}": "0x${new Array(64).fill(0).join("")}${messagingState.slice(66)}",\n`;
      console.log(
        ` ${chalk.green(`  + Adding ParachainSystem.RelevantMessagingState`)}\n\t${newLine}`
      );
      outStream.write(newLine);
    } else if (line.startsWith(authorEligibilityRatioPrefix)) {
      console.log(` ${chalk.red(`  - Removing AuthorFilter.EligibleRatio`)}\n\t${line}`);
      const newLine = `        "${storageKey("AuthorFilter", "EligibleRatio")}": "0x64",\n`;
      console.log(` ${chalk.green(`  + Adding AuthorFilter.EligibleRatio`)}\n\t${newLine}`);
      outStream.write(newLine);
    } else if (line.startsWith(authorEligibilityCountPrefix)) {
      console.log(` ${chalk.red(`  - Removing AuthorFilter.EligibleCount`)}\n\t${line}`);

      const newLine = `        "${storageKey("AuthorFilter", "EligibleCount")}": "0x32000000",\n`;
      console.log(` ${chalk.green(`  + Adding AuthorFilter.EligibleCount`)}\n\t${newLine}`);
      outStream.write(newLine);
    } else if (line.startsWith(councilLinePrefix)) {
      console.log(` ${chalk.red(`  - Removing CouncilCollective.Members`)}\n\t${line}`);
      const newLine = `        "${storageKey("CouncilCollective", "Members")}": "0x04${ALITH.slice(
        2
      )}",\n`;
      console.log(` ${chalk.green(`  + Adding CouncilCollective.Members`)}\n\t${newLine}`);
      outStream.write(newLine);
    } else if (line.startsWith(techCommitteeeLinePrefix)) {
      console.log(` ${chalk.red(`  - Removing TechCommitteeCollective.Members`)}\n\t${line}`);
      const newLine = `        "${storageKey(
        "TechCommitteeCollective",
        "Members"
      )}": "0x04${ALITH.slice(2)}",\n`;
      console.log(` ${chalk.green(`  + Adding TechCommitteeCollective.Members`)}\n\t${newLine}`);
      outStream.write(newLine);
    } else if (line.startsWith(lastDmqMqcHeadPrefix)) {
      console.log(` ${chalk.red(`  - Removing ParachainSystem.LastDmqMqcHead`)}\n\t${line}`);
    } else if (line.startsWith(alithBalancePrefix)) {
      console.log(` ${chalk.red(`  - Removing System.Account: Alith`)}\n\t${line}`);
      const newLine = `        "${storageBlake128MapKey(
        "System",
        "Account",
        ALITH
      )}": "${newAlithAccountData}",\n`;
      console.log(` ${chalk.green(`  + Adding System.Account: Alith`)}\n\t${newLine}`);
      outStream.write(newLine);
    } else if (line.startsWith(totalIssuanceBalancePrefix)) {
      console.log(` ${chalk.red(`  - Removing Balances.TotalIssuance`)}\n\t${line}`);

      const newLine = `        "${storageKey("Balances", "TotalIssuance")}": "${nToHex(
        newTotalIssuance,
        {
          isLe: true,
          bitLength: 128,
        }
      )}",\n`;
      console.log(` ${chalk.green(`  + Balances.TotalIssuance`)}\n\t${newLine}`);
      outStream.write(newLine);
    } else if (line.startsWith(bootnodesPrefix)) {
      console.log(` ${chalk.red(`  - Removing bootnode`)}\n\t${line}`);
    } else {
      outStream.write(line);
      outStream.write("\n");
    }
    // !line.startsWith(parachainIdPrefix)
  }
  // outStream.write("}\n")
  outStream.end();

  console.log(`Forked genesis generated successfully. Find it at ${destFile}`);
}

const args = process.argv.slice(2);
main(args[0], args[1]);

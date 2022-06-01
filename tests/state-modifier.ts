import fs from "fs";
import readline from "readline";
import chalk from "chalk";

import { xxhashAsU8a, blake2AsU8a } from "@polkadot/util-crypto";
import { u8aConcat, u8aToHex } from "@polkadot/util";

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
  const BOB = "0x3Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0";
  const BOB_SESSION = "0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48";

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
  const alithBalance = `        "${storageBlake128MapKey("System", "Account", ALITH)}`;
  const bobBalance = `        "${storageBlake128MapKey("System", "Account", BOB)}`;

  // List all the collator author mapping
  const collatorAuthorMapping = {};

  // First pass
  let selectedCollator = null;
  for await (const line of rl1) {
    if (line.startsWith(collatorLinePrefix)) {
      // First collator we found, it will be our target for Alice
      if (!selectedCollator) {
        selectedCollator = `0x${line.split('"')[3].slice(-40)}`;
        console.log(`Using account ${selectedCollator} as alice collator`);
      }
    }
    if (line.startsWith(authorLinePrefix)) {
      // First collator we found, it will be our target for Alice
      if (!selectedCollator) {
        const collator = line.split('"')[3].slice(0, 42);
        const collatorMappingKey = line.split('"')[1];
        collatorAuthorMapping[collator] = collatorMappingKey;
      }
    }
    if (line.startsWith(revelentMessagingStatePrefix)) {
      messagingState = line.split('"')[3];
    }
  }

  // List all the collator author mapping
  console.log(
    `Using account ${selectedCollator} as alice collator, session ${collatorAuthorMapping[selectedCollator]}`
  );

  if (!selectedCollator) {
    console.log(`Couldn't find collator with prefix ${authorLinePrefix}`);
    return;
  }
  const collatorMappingKey = collatorAuthorMapping[selectedCollator];

  if (!messagingState) {
    console.log(`Couldn't find messaging state with prefix ${revelentMessagingStatePrefix}`);
    return;
  }

  const in2Stream = fs.createReadStream(inputFile, "utf8");
  const rl2 = readline.createInterface({
    input: in2Stream,
    crlfDelay: Infinity,
  });
  const outStream = fs.createWriteStream(destFile, { encoding: "utf8" });

  const authorMappingPrefix = `        "${collatorMappingKey}"`;

  for await (const line of rl2) {
    if (line.startsWith(`      "top"`)) {
      outStream.write(line);
      console.log("found top");
      const newLine = `        "${storageBlake128MapKey(
        "AuthorMapping",
        "MappingWithDeposit",
        ALITH_SESSION
      )}": "${selectedCollator}000010632d5ec76b0500000000000000${ALITH_SESSION.slice(2)}",\n`;
      console.log(` ${chalk.green(`+ Adding session`)}\n\t${newLine}`);
      outStream.write(newLine);

      // outStream.write(
      //   `        "${storageBlake128MapKey(
      //     "AuthorMapping",
      //     "MappingWithDeposit",
      //     BOB_SESSION
      //   )}": "${selectedCollator}000010632d5ec76b0500000000000000${BOB_SESSION.slice(2)}",\n`
      // );
      // outStream.write(
      //   `        "${storageKey(
      //     "ParachainSystem",
      //     "LastDmqMqcHead"
      //   )}": "${collator}000010632d5ec76b0500000000000000",\n`
      // );
      outStream.write(
        `        "${storageKey("ParachainSystem", "RelevantMessagingState")}": "0x${new Array(64)
          .fill(0)
          .join("")}${messagingState.slice(66)}",\n`
      );
      outStream.write(`        "${storageKey("AuthorFilter", "EligibleRatio")}": "0x64",\n`);
      outStream.write(`        "${storageKey("AuthorFilter", "EligibleCount")}": "0x32000000",\n`);
      outStream.write(
        `        "${storageKey("CouncilCollective", "Members")}": "0x04${ALITH.slice(2)}",\n`
      );
      outStream.write(
        `        "${storageKey("TechCommitteeCollective", "Members")}": "0x04${ALITH.slice(2)}",\n`
      );
      // outStream.write(
      //   `        "${storageKey("ParachainInfo", "ParachainId")}": "0xe8030000",\n`
      // );
      outStream.write(
        `        "${storageBlake128MapKey(
          "System",
          "Account",
          ALITH
        )}": "0x00000000000000000000000000000000d699d3ded12e14d6e701000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",\n`
      );
      outStream.write(
        `        "${storageBlake128MapKey(
          "System",
          "Account",
          BOB
        )}": "0x00000000000000000000000000000000d699d3ded12e14d6e701000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",\n`
      );
    } else if (line.startsWith(authorMappingPrefix)) {
      console.log(` ${chalk.red(`- Removing session`)}\n\t${line}`);
    } else if (
      !line.startsWith(revelentMessagingStatePrefix) &&
      !line.startsWith(authorEligibilityRatioPrefix) &&
      !line.startsWith(authorEligibilityCountPrefix) &&
      !line.startsWith(councilLinePrefix) &&
      !line.startsWith(techCommitteeeLinePrefix) &&
      !line.startsWith(authorMappingPrefix) &&
      // !line.startsWith(parachainIdPrefix) &&
      !line.startsWith(lastDmqMqcHeadPrefix) &&
      !line.startsWith(alithBalance) &&
      !line.startsWith(bobBalance)
    ) {
      outStream.write(line);
      outStream.write("\n");
    }
  }
  // outStream.write("}\n")
  outStream.end();

  console.log(`Forked genesis generated successfully. Find it at ${destFile}`);
}

const args = process.argv.slice(2);
main(args[0], args[1]);

import fs from "fs";
import readline from "readline";
import chalk from "chalk";

import { xxhashAsU8a, blake2AsU8a } from "@polkadot/util-crypto";
import { u8aConcat, u8aToHex, hexToBigInt, nToHex, bnToHex } from "@polkadot/util";
import { DOT_ASSET_ID, USDT_ASSET_ID } from "./fork-tests/staticData";

const storageKey = (module, name) => {
  return u8aToHex(u8aConcat(xxhashAsU8a(module, 128), xxhashAsU8a(name, 128)));
};

const storageBlake128MapKey = (module, name, key) => {
  return u8aToHex(
    u8aConcat(xxhashAsU8a(module, 128), xxhashAsU8a(name, 128), blake2AsU8a(key, 128), key)
  );
};

const storageBlake128DoubleMapKey = (module, name, [key1, key2]) => {
  return u8aToHex(
    u8aConcat(
      xxhashAsU8a(module, 128),
      xxhashAsU8a(name, 128),
      blake2AsU8a(key1, 128),
      key1,
      blake2AsU8a(key2, 128),
      key2
    )
  );
};

// const ALITH_PRIV_KEY = "0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133";
// const BOB_PRIV_KEY = "0x8075991ce870b93a8870eca0c0f91913d12f47948ca0fd25b49c6fa7cdbeee8b";

async function main(inputFile: string, outputFile?: string) {
  const ALITH = "0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac";
  const ALITH_SESSION = "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";

  console.log(chalk.blueBright(`  * Preparing Alith: ${ALITH} session ${ALITH_SESSION}`));

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

  let messagingState: string = "";
  const collatorLinePrefix = `        "${storageKey("ParachainStaking", "SelectedCandidates")}`;
  const roundLinePrefix = `        "${storageKey("ParachainStaking", "Round")}`;
  const orbiterLinePrefix = `        "${storageKey("MoonbeamOrbiters", "CollatorsPool")}`;
  const nimbusBlockNumberPrefix = `        "${storageKey("AuthorInherent", "HighestSlotSeen")}`;
  const authorLinePrefix = `        "${storageKey("AuthorMapping", "MappingWithDeposit")}`;
  const revelentMessagingStatePrefix = `        "${storageKey(
    "ParachainSystem",
    "RelevantMessagingState"
  )}`;
  const validationDataPrefix = `        "${storageKey("ParachainSystem", "ValidationData")}`;
  const lastRelayChainBlockNumberPrefix = `        "${storageKey(
    "ParachainSystem",
    "LastRelayChainBlockNumber"
  )}`;
  const authorEligibilityRatioPrefix = `        "${storageKey("AuthorFilter", "EligibleRatio")}`;
  const authorEligibilityCountPrefix = `        "${storageKey("AuthorFilter", "EligibleCount")}`;
  const councilLinePrefix = `        "${storageKey("CouncilCollective", "Members")}`;
  const techCommitteeeLinePrefix = `        "${storageKey("TechCommitteeCollective", "Members")}`;
  const highestSlotSeenPrefix = `        "${storageKey("AuthorInherent", "HighestSlotSeen")}`;
  const parachainIdPrefix = `        "${storageKey("ParachainInfo", "ParachainId")}`;
  const lastDmqMqcHeadPrefix = `        "${storageKey("ParachainSystem", "LastDmqMqcHead")}`;
  const alithBalancePrefix = `        "${storageBlake128MapKey("System", "Account", ALITH)}`;
  const totalIssuanceBalancePrefix = `        "${storageKey("Balances", "TotalIssuance")}`;
  const assetsBalancePrefix = `        "${storageKey("Assets", "Account")}`;
  const inboundXcmpMessagesPrefix = `        "${storageKey("XcmpQueue", "InboundXcmpMessages")}`;
  const inboundXcmpStatusPrefix = `        "${storageKey("XcmpQueue", "InboundXcmpStatus")}`;
  const outboundXcmpMessagesPrefix = `        "${storageKey("XcmpQueue", "OutboundXcmpMessages")}`;
  const outboundXcmpStatusPrefix = `        "${storageKey("XcmpQueue", "OutboundXcmpStatus")}`;
  const overweightPrefix = `        "${storageKey("XcmpQueue", "Overweight")}`;
  const overweightCountPrefix = `        "${storageKey("XcmpQueue", "OverweightCount")}`;
  const signalMessagesPrefix = `        "${storageKey("XcmpQueue", "SignalMessages")}`;

  const bootnodesPrefix = `    "/`;

  // List all the collator author mapping
  const authorMappingLines = {};

  // First pass
  let collators: string[] = [];
  let orbiters: string[] = [];
  // let selectedCollator = null;
  let totalIssuance: bigint = 0n;
  let validationData: string = "";
  let roundNumber: bigint = 0n;
  let alithAccountData;
  for await (const line of rl1) {
    if (line.startsWith(collatorLinePrefix)) {
      const data = line.split('"')[3].slice(2);
      // the data contains arbitrary size as bytes at the beginning so we parse from the end;
      for (let i = data.length; i > 40; i -= 40) {
        collators.push(`0x${data.slice(i - 40, i)}`);
      }
    }
    if (line.startsWith(orbiterLinePrefix)) {
      orbiters.push(`0x${line.split('"')[1].slice(-40)}`);
    }
    if (line.startsWith(authorLinePrefix)) {
      const collator = line.split('"')[3].slice(0, 42);
      authorMappingLines[collator] = line;
    }
    if (line.startsWith(revelentMessagingStatePrefix)) {
      messagingState = line.split('"')[3];
    }
    if (line.startsWith(validationDataPrefix)) {
      validationData = line.split('"')[3];
    }
    if (line.startsWith(totalIssuanceBalancePrefix)) {
      totalIssuance = hexToBigInt(line.split('"')[3], { isLe: true });
    }
    if (line.startsWith(alithBalancePrefix)) {
      alithAccountData = line.split('"')[3];
    }
    if (line.startsWith(roundLinePrefix)) {
      roundNumber = hexToBigInt(line.split('"')[3].slice(0, 2 + 8), { isLe: true });
    }
  }
  // We make sure the collator is not an orbiter
  const selectedCollator =
    collators.find((c) => !orbiters.includes(c) && authorMappingLines[c]) || "";
  console.log(
    chalk.blueBright(
      `  *  Found collator: ${selectedCollator} session 0x${authorMappingLines[selectedCollator]
        .split('"')[1]
        .slice(-64)}`
    )
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
  const amount = nToHex(15_000_000_000_000, { isLe: true });
  const newAlithTokenBalanceData = "0x" + amount.slice(2).padEnd(35, "0") + "1";

  const in2Stream = fs.createReadStream(inputFile, "utf8");
  const rl2 = readline.createInterface({
    input: in2Stream,
    crlfDelay: Infinity,
  });
  const outStream = fs.createWriteStream(destFile, { encoding: "utf8" });

  const selectedAuthorMappingPrefix = `        "${selectedCollatorMappingKey}"`;
  const selectedNimbusLookup = storageBlake128MapKey(
    "AuthorMapping",
    "NimbusLookup",
    selectedCollator
  );
  const selectedNimbusLookupPrefix = `        "${selectedNimbusLookup}"`;
  const alithAuthorMappingPrefix = `        "${storageBlake128MapKey(
    "AuthorMapping",
    "MappingWithDeposit",
    ALITH_SESSION
  )}`;

  let injected = false;

  for await (const line of rl2) {
    if (line.startsWith(alithAuthorMappingPrefix)) {
      console.log(
        ` ${chalk.red(
          `  - Removing (Extra) AuthorMapping.MappingWithDeposit ${ALITH_SESSION}`
        )}\n\t${line}`
      );
    } else if (line.startsWith(nimbusBlockNumberPrefix)) {
      console.log(` ${chalk.red(`  - Removing AuthorInherent.HighestSlotSeen`)}\n\t${line}`);
      let newLine = `        "${storageKey("AuthorInherent", "HighestSlotSeen")}": "0x00000000",\n`;
      console.log(` ${chalk.green(`  + Adding AuthorInherent.HighestSlotSeen`)}\n\t${newLine}`);
      outStream.write(newLine);
    } else if (line.startsWith(roundLinePrefix)) {
      console.log(` ${chalk.red(`  - Removing ParachainStaking.Round`)}\n\t${line}`);
      const roundLength = 100; // blocks (more than collators, short to make it faster)
      // Using the real round number;
      // const roundNumber = nToHex(1, { isLe: true, bitLength: 32 }).slice(2);
      // see https://github.com/polkadot-js/api/issues/5262
      const firstBlock = "0".padStart((32 / 8) * 2, "0");
      const length = nToHex(roundLength, { isLe: true, bitLength: 32 }).slice(2);
      let newLine = `        "${storageKey("ParachainStaking", "Round")}": "${nToHex(
        Number(roundNumber),
        { isLe: true, bitLength: 32 }
      )}${firstBlock}${length}",\n`;
      console.log(
        ` ${chalk.green(`  + Adding ParachainStaking.Round (${roundLength} blocks)`)}\n\t${newLine}`
      );
      outStream.write(newLine);
    } else if (line.startsWith(selectedAuthorMappingPrefix)) {
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
    } else if (line.startsWith(selectedNimbusLookupPrefix)) {
      console.log(
        ` ${chalk.red(`  - Removing AuthorMapping.NimbusLookup ${selectedCollator}`)}\n\t${line}`
      );
      let newLine = `${selectedNimbusLookupPrefix}: "${ALITH_SESSION}",\n`;
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

      const newLine = `        "${storageKey("AuthorFilter", "EligibleCount")}": "0xFF000000",\n`;
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
    } else if (line.startsWith(highestSlotSeenPrefix)) {
      console.log(` ${chalk.red(`  - Removing AuthorInherent.HighestSlotSeen`)}\n\t${line}`);
      const newLine = `        "${storageKey(
        "AuthorInherent",
        "HighestSlotSeen"
      )}": "0x00000000",\n`;
      console.log(` ${chalk.green(`  + Adding AuthorInherent.HighestSlotSeen`)}\n\t${newLine}`);
      outStream.write(newLine);
    } else if (line.startsWith(lastDmqMqcHeadPrefix)) {
      console.log(` ${chalk.red(`  - Removing ParachainSystem.LastDmqMqcHead`)}\n\t${line}`);
    } else if (line.startsWith(lastRelayChainBlockNumberPrefix)) {
      console.log(
        ` ${chalk.red(`  - Removing ParachainSystem.LastRelayChainBlockNumber`)}\n\t${line}`
      );
      const newLine = `        "${storageKey(
        "ParachainSystem",
        "LastRelayChainBlockNumber"
      )}": "0x00000000",\n`;
      console.log(
        ` ${chalk.green(`  + Adding ParachainSystem.LastRelayChainBlockNumber`)}\n\t${newLine}`
      );
      outStream.write(newLine);
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
      console.log(` ${chalk.green(`  + Adding Balances.TotalIssuance`)}\n\t${newLine}`);
      outStream.write(newLine);
    } else if (line.startsWith(validationDataPrefix)) {
      console.log(` ${chalk.red(`  - Removing ParachainSystem.ValidationData`)}\n\t${line}`);

      const head = validationData.slice(0, -(8 + 64 + 8));
      const relayParentNumber = "00000000";
      const relayParentStorageRoot = validationData.slice(-(8 + 64), -8);
      const maxPovSize = validationData.slice(-8);
      const newLine = `        "${storageKey(
        "ParachainSystem",
        "ValidationData"
      )}": "${head}${relayParentNumber}${relayParentStorageRoot}${maxPovSize}",\n`;
      console.log(` ${chalk.green(`  + Adding ParachainSystem.ValidationData`)}\n\t${newLine}`);
      outStream.write(newLine);
    } else if (line.startsWith(bootnodesPrefix)) {
      console.log(` ${chalk.red(`  - Removing bootnode`)}\n\t${line}`);
    } else if (line.startsWith(inboundXcmpMessagesPrefix)) {
      console.log(` ${chalk.red(`  - Removing inboundXcmpMessagesPrefix`)}\n\t${line}`);
    } else if (line.startsWith(inboundXcmpStatusPrefix)) {
      console.log(` ${chalk.red(`  - Removing inboundXcmpStatusPrefix`)}\n\t${line}`);
    } else if (line.startsWith(outboundXcmpMessagesPrefix)) {
      console.log(` ${chalk.red(`  - Removing outboundXcmpMessagesPrefix`)}\n\t${line}`);
    } else if (line.startsWith(outboundXcmpStatusPrefix)) {
      console.log(` ${chalk.red(`  - Removing outboundXcmpStatusPrefix`)}\n\t${line}`);
    } else if (line.startsWith(overweightPrefix)) {
      console.log(` ${chalk.red(`  - Removing overweightPrefix`)}\n\t${line}`);
    } else if (line.startsWith(overweightCountPrefix)) {
      console.log(` ${chalk.red(`  - Removing overweightCountPrefix`)}\n\t${line}`);
    } else if (line.startsWith(signalMessagesPrefix)) {
      console.log(` ${chalk.red(`  - Removing signalMessagesPrefix`)}\n\t${line}`);
    } else if (line.startsWith(assetsBalancePrefix)) {
      if (!injected) {
        injected = true;

        const dotLine = `        "${storageBlake128DoubleMapKey("Assets", "Account", [
          bnToHex(BigInt(DOT_ASSET_ID), { isLe: true, bitLength: 128 }),
          ALITH,
        ])}": "${newAlithTokenBalanceData}",\n`;
        console.log(` ${chalk.green(`  + Adding Assets.Account Alith DOT`)}\n\t${dotLine}`);
        outStream.write(dotLine);

        const usdtLine = `        "${storageBlake128DoubleMapKey("Assets", "Account", [
          bnToHex(BigInt(USDT_ASSET_ID), { isLe: true, bitLength: 128 }),
          ALITH,
        ])}": "${newAlithTokenBalanceData}",\n`;
        console.log(` ${chalk.green(`  + Adding Assets.Account Alith USDT`)}\n\t${usdtLine}`);
        outStream.write(usdtLine);
      } else {
        outStream.write(line);
        outStream.write("\n");
      }
    } else {
      outStream.write(line);
      outStream.write("\n");
    }
    // !line.startsWith(parachainIdPrefix)
  }
  outStream.end();

  console.log(`Forked genesis generated successfully. Find it at ${destFile}`);
}

const args = process.argv.slice(2);
main(args[0], args[1]);

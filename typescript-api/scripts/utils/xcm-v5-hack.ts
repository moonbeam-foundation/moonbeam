import { existsSync, readFileSync, writeFileSync } from "node:fs";
import chalk from "chalk";

export function hackTypeReplacement(
  filePath: string,
  oldType: string,
  newType: string,
  expectedCount: number
) {
  if (!existsSync(filePath)) {
    console.error(chalk.red(`Error: File ${filePath} does not exist.`));
    process.exit(1);
  }
  const content = readFileSync(filePath, "utf-8");

  console.log("XCM v5 hack: updating ", filePath);
  logMatchingLines(filePath, "@name StagingXcmV5Junction ");
  console.log("Line above should say", oldType);

  const regex = new RegExp(oldType, "g");
  const matches = content.match(regex);
  const count = matches ? matches.length : 0;
  if (count !== expectedCount) {
    // This check is to ensure we don't accidentally replace more than needed, if there is a Lookup777 for example,
    // we only want to replace Lookup77
    console.error(
      chalk.red(
        `Error: Expected ${expectedCount} occurrences of "${oldType}" in ${filePath} but found ${count}. Aborting hack.`
      )
    );
    process.exit(1);
  }
  const newContent = content.replace(regex, newType);
  writeFileSync(filePath, newContent);
  console.log(
    chalk.green(
      `Successfully replaced ${count} occurrences of "${oldType}" with "${newType}" in ${filePath}`
    )
  );
}

function logMatchingLines(filePath: string, substring: string) {
  const content = readFileSync(filePath, "utf-8");
  const lines = content.split(/\r?\n/);
  for (const line of lines) {
    if (line.includes(substring)) {
      console.log(`Found matching line in ${filePath}: ${line}`);
    }
  }
}

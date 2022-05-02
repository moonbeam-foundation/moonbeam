import yargs from "yargs";
import chalk from "chalk";
import fs from "fs";

const argv = yargs(process.argv.slice(2))
  .usage("Usage: $0")
  .version("1.0.0")
  .options({
    log: {
      type: "string",
      description: "log file to parse containing migration",
      demandOption: true,
    },
  }).argv;

const main = async () => {
  const lines = fs.readFileSync(argv.log).toString().split(/\r?\n/);
  let hasMigrationStarted = false;
  let proposingLine;
  let storageProofLine;
  let errorLines = [];
  for (const line of lines) {
    if (/Performing on_runtime_upgrade/g.test(line)) {
      hasMigrationStarted = true;
      continue;
    }
    if (!hasMigrationStarted) {
      continue;
    }
    if (/ ERROR | WARN /g.test(line)) {
      errorLines.push(line);
    }
    if (/proposing at/g.test(line)) {
      proposingLine = line;
    }
    if (/storage_proof/g.test(line)) {
      storageProofLine = line;
    }
    if (/Compressed PoV size/g.test(line)) {
      console.log(`Migration ${chalk.green("executed")}`);
      const compressedPov = parseInt(/Compressed PoV size: ([0-9\.]*)kb/g.exec(line)?.[1]);
      const storageProof = parseInt(/storage_proof: ([0-9\.]*)kb/g.exec(storageProofLine)?.[1]);
      const executionTime = parseInt(
        /proposing at [0-9]* \(([0-9]*) ms\)/g.exec(proposingLine)?.[1]
      );

      console.log(
        `Compressed PoV: ${
          compressedPov < 1000
            ? chalk.green(compressedPov)
            : compressedPov < 2000
            ? chalk.yellow(compressedPov)
            : chalk.red(compressedPov)
        } kb  (storage_proof: ${
          storageProof < 1500
            ? chalk.green(storageProof)
            : storageProof < 3000
            ? chalk.yellow(storageProof)
            : chalk.red(storageProof)
        } kb)`
      );
      console.log(
        `Execution time: ${
          executionTime < 100
            ? chalk.green(executionTime)
            : executionTime < 300
            ? chalk.yellow(executionTime)
            : chalk.red(executionTime)
        } ms`
      );

      for (const errorLine of errorLines) {
        console.log(errorLine);
      }
      hasMigrationStarted = false;
      proposingLine;
      storageProofLine;
      errorLines = [];
      console.log(`=============================`);
    }
  }
};

main().catch((e) => {
  console.trace(e);
  process.exit(1);
});

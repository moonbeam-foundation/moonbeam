import yargs from "yargs";
import chalk from "chalk";
import fs from "node:fs";

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
  let migrationTime = "";
  let migrations = [];
  let errorLines = [];
  for (const line of lines) {
    if (/Performing on_runtime_upgrade/g.test(line)) {
      hasMigrationStarted = true;
      migrationTime = / ([^\ ]*)  WARN tokio-runtime-worker/g.exec(line)[1];
      continue;
    }
    if (!hasMigrationStarted) {
      continue;
    }
    if (/ ERROR | WARN /g.test(line)) {
      errorLines.push(line);
    }
    if (/performing migration/g.test(line)) {
      migrations.push(/performing migration (.*)/g.exec(line)[1]);
    }
    if (/proposing at/g.test(line)) {
      proposingLine = line;
    }
    if (/storage_proof/g.test(line)) {
      storageProofLine = line;
    }
    if (/Compressed PoV size/g.test(line)) {
      console.log(`Migration ${chalk.green("executed")}: ${migrationTime}`);
      const compressedPov = parseInt(/Compressed PoV size: ([0-9\.]*)kb/g.exec(line)?.[1]);
      const storageProof = parseInt(/storage_proof: ([0-9\.]*)kb/g.exec(storageProofLine)?.[1]);
      const executionTime = parseInt(
        /proposing at [0-9]* \(([0-9]*) ms\)/g.exec(proposingLine)?.[1]
      );

      migrations.forEach((line) => {
        console.log(
          `  - ${chalk.yellow(line.split(" ")[0])} (${chalk.grey(
            line
              .split(" ")
              .slice(1)
              .filter((l) => l.length > 0)
              .join(" ")
          )})`
        );
      });

      console.log(
        `Compressed PoV: ${
          compressedPov < 1000
            ? chalk.green(compressedPov.toString())
            : compressedPov < 2000
            ? chalk.yellow(compressedPov.toString())
            : chalk.red(compressedPov.toString())
        } kb  (storage_proof: ${
          storageProof < 1500
            ? chalk.green(storageProof.toString())
            : storageProof < 3000
            ? chalk.yellow(storageProof.toString())
            : chalk.red(storageProof.toString())
        } kb)`
      );
      console.log(
        `Execution time: ${
          executionTime < 100
            ? chalk.green(executionTime.toString())
            : executionTime < 300
            ? chalk.yellow(executionTime.toString())
            : chalk.red(executionTime.toString())
        } ms`
      );

      for (const errorLine of errorLines) {
        console.log(errorLine);
      }
      hasMigrationStarted = false;
      proposingLine;
      storageProofLine;
      errorLines = [];
      migrations = [];
      console.log(`=============================`);
    }
  }
};

main().catch((e) => {
  console.trace(e);
  process.exit(1);
});

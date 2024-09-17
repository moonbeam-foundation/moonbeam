// Create a function that navigates to a sibling package directory
// and from there executes one of the package scripts named "trace"

import { exec, spawn } from "child_process";
import { promisify } from "util";
import { join, dirname } from "path";
import { fileURLToPath } from "url";
import { readFileSync } from "fs";

const execAsync = promisify(exec);

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const executeScript = async (relativeDir: string, command: string) => {
  const targetDir = join(__dirname, relativeDir);
  const pkgJsonPath = join(targetDir, "package.json");
  try {
    const pkgJson = JSON.parse(readFileSync(pkgJsonPath, "utf-8"));
    console.log(`Executing ${command} script in package ${relativeDir}`);
    const { stdout, stderr } = await execAsync(command, { cwd: targetDir });
    // If stdout includes Done, print the line including Done in the string
    if (stdout.includes("Done in "))
      console.log(
        `${stdout
          .trim()
          .split("\n")
          .find((line) => line.includes("Done"))} ✅`
      );
    if (stdout) console.log(`${stdout}`);
    if (stderr) console.error(`stderr: ${stderr}`);
  } catch (e) {
    console.error(`Error executing ${command} script in package ${relativeDir}`);
    console.error(e);
  }
};

// A function that checks that ../../target/release/moonbeam binary exists
const checkBinary = async () => {
  try {
    const { stdout, stderr } = await execAsync("ls ../target/release/moonbeam");
    if (stdout) console.log(`${stdout}`);
    if (stderr) console.error(`stderr: ${stderr}`);
  } catch (e) {
    console.error("Moonbeam binary missing, please build it first using `cargo build --release`");
    // console.error(e);
  }
};

// A function that spawns a child running the moonbeam binary in the background and returns the process
const startMoonbeam = () => {
  const moonbeam = spawn(
    "../target/release/moonbeam",
    ["--tmp", "--chain=moonbase-local", "--rpc-port=9944"],
    {
      detached: true,
      stdio: "ignore",
    }
  );
  return moonbeam;
};

const executeUpdateAPIScript = async () => {
  await checkBinary();
  await executeScript("../../moonbeam-types-bundle", "pnpm add @polkadot/api@latest");
  await executeScript("../../typescript-api", "pnpm add @polkadot/api@latest");
  await executeScript("../../typescript-api", "pnpm add @polkadot/typegen@latest");
  await executeScript("../../typescript-api", "pnpm build");
  // Install packages
  await executeScript("../../moonbeam-types-bundle", "pnpm install");
  const node = startMoonbeam();
  // Wait 5 seconds
  await new Promise((resolve) => setTimeout(resolve, 5000));
  // Generate types
  await executeScript("../../typescript-api", "pnpm load:meta");
  await executeScript("../../typescript-api", "pnpm load:meta:local");
  // Kill the node
  node.kill();
  console.log("Done updating Typescript API ✅");
  // Installing new types for Test package
  await executeScript("..", "pnpm install");
};

executeUpdateAPIScript();

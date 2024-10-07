import { exec, spawn } from "child_process";
import { promisify } from "util";
import { join, dirname } from "path";
import { fileURLToPath } from "url";
import { readFileSync, write, writeFileSync } from "fs";


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
    // if (stdout) console.log(`${stdout}`);
    if (stderr) console.error(`stderr: ${stderr}`);
  } catch (e) {
    console.error(`Error executing ${command} script in package ${relativeDir}`);
    console.error(e);
  }
};

const writeFile = async (relativeDir: string, fileName: string, data: string) => {
  const targetDir = join(__dirname, relativeDir);
  const filePath = join(targetDir, fileName);
  writeFileSync(filePath, data, { flag: "w" });
}

const checkBinary = async () => {
  try {
    const { stdout, stderr } = await execAsync("ls ../target/release/moonbeam");
    if (stderr) console.error(`stderr: ${stderr}`);
  } catch (e) {
    console.error("Moonbeam binary missing, please build it first using `cargo build --release`");
  }
};

const startNode = (network: string, port: string) => {
  console.log(`Starting ${network.toUpperCase()} node at port `, port);
  const node = spawn(
    "../target/release/moonbeam",
    [
      `--alice`,
      `--chain=${network}-local`,
      `--rpc-port=${port}`,
      `--no-hardware-benchmarks`,
      `--unsafe-force-node-key-generation`,
      `--wasm-execution=interpreted-i-know-what-i-do`,
      `--no-telemetry`,
      `--no-prometheus`,
      "--tmp",
    ],
    {
      detached: true,
      stdio: "ignore",
    }
  );
  return node;
};

const scrapeMetadata = async (network: string, port: string) => {
  const metadata = await fetch(`http://localhost:${port}`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      id: "1",
      jsonrpc: "2.0",
      method: "state_getMetadata",
      params: [],
    }),
  });

  const metadataJson = await metadata.json();
  writeFile(`../../typescript-api`, `metadata-${network}.json`, JSON.stringify(metadataJson));
}

const executeUpdateAPIScript = async () => {
  await checkBinary();

  // Bundle types
  await executeScript("../../moonbeam-types-bundle", "pnpm i");
  await executeScript("../../moonbeam-types-bundle", "pnpm build");
  await executeScript("../../moonbeam-types-bundle", "pnpm fmt:fix");

  // Generate types
  console.log("Extracting metadata for all runtimes...");
  const networks = ["moonbase", "moonriver", "moonbeam"];
  const port = "9933";
  for (const network of networks) {
    const node = startNode(network, port);
    await new Promise((resolve) => setTimeout(resolve, 5000));
    await scrapeMetadata(network, port);
    node.kill();
    console.log(`Metadata for ${network} saved ✅`);
  }

  // Generate meta & defs
  await executeScript("../../typescript-api", "pnpm generate:defs");
  await executeScript("../../typescript-api", "pnpm generate:meta");

  // Build the API
  await executeScript("../../typescript-api", "pnpm build");

  // Fix formatting
  await executeScript("../../typescript-api", "pnpm fmt:fix");

  // Install new types for Test package
  await executeScript("..", "pnpm install");

  console.log("Done updating Typescript API ✅");
};

executeUpdateAPIScript();

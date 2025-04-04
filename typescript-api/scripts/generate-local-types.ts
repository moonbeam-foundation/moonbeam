import { spawn } from "node:child_process";
import type { ChildProcess } from "node:child_process";
import { existsSync, createWriteStream } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import fetch from "node-fetch";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

async function checkMoonbeamBinary(): Promise<void> {
  const moonbeamPath = join(__dirname, "../../target/release/moonbeam");
  if (!existsSync(moonbeamPath)) {
    throw new Error("Missing moonbeam binary. Please run cargo build --release");
  }
}

async function installPackages(): Promise<void> {
  console.log("Installing Packages");
  await runCommand("pnpm", ["i"]);
}

async function startMoonbeamNode(): Promise<ChildProcess> {
  console.log("Starting moonbeam node");
  const moonbeamPath = join(__dirname, "../../target/release/moonbeam");
  const node = spawn(moonbeamPath, ["--tmp", "--chain=moonbase-dev", "--rpc-port=9933"], {
    stdio: ["ignore", "pipe", "pipe"]
  });

  // Pipe output to a log file
  const logStream = createWriteStream("/tmp/node-start.log");
  node.stdout?.pipe(logStream);
  node.stderr?.pipe(logStream);

  return node;
}

async function waitForNodeStart(): Promise<void> {
  console.log("Waiting for node to start...");
  const maxAttempts = 100; // 5 seconds total with 100ms intervals
  let attempts = 0;

  while (attempts < maxAttempts) {
    try {
      const response = await fetch("http://localhost:9933", {
        method: "POST",
        headers: {
          "Content-Type": "application/json"
        },
        body: JSON.stringify({
          jsonrpc: "2.0",
          method: "system_health",
          params: [],
          id: 1
        })
      });

      if (response.ok) {
        console.log("Node is ready!");
        return;
      }
    } catch (error) {
      // Ignore connection errors while node is starting
    }

    await new Promise((resolve) => setTimeout(resolve, 100));
    attempts++;
  }

  throw new Error("Node failed to start within 10 seconds");
}

async function generateTypes(): Promise<void> {
  console.log("Generating types...(10s)");
  await new Promise((resolve) => setTimeout(resolve, 1000));

  const commands = [
    ["load:meta"],
    ["load:meta:local"],
    ["generate:defs"],
    ["generate:meta"],
    ["fmt:fix"]
  ];

  for (const command of commands) {
    await runCommand("pnpm", command);
  }
}

async function runCommand(command: string, args: string[]): Promise<void> {
  return new Promise((resolve, reject) => {
    const process = spawn(command, args, { stdio: "inherit" });
    process.on("close", (code) => {
      if (code === 0) {
        resolve();
      } else {
        reject(new Error(`Command ${command} failed with code ${code}`));
      }
    });
  });
}

async function main() {
  try {
    await checkMoonbeamBinary();
    await installPackages();

    const node = await startMoonbeamNode();
    await waitForNodeStart();
    await generateTypes();

    node.kill();
    console.log("Done :)");
    process.exit(0);
  } catch (error) {
    console.error("Error:", error);
    process.exit(1);
  }
}

// Handle process termination
process.on("SIGINT", () => {
  console.log("Received SIGINT. Cleaning up...");
  process.exit(0);
});

process.on("SIGTERM", () => {
  console.log("Received SIGTERM. Cleaning up...");
  process.exit(0);
});

main();

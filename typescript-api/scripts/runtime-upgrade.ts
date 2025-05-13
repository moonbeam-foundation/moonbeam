import { spawn, execSync } from "node:child_process";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import axios from "axios";
import { hackXcmV5Support } from "./utils/xcm-v5-hack";

const CHAINS = ["moonbase", "moonriver", "moonbeam"];

async function startNode(chain: string): Promise<number> {
  const args = [
    "--no-hardware-benchmarks",
    "--unsafe-force-node-key-generation",
    "--no-telemetry",
    "--no-prometheus",
    "--alice",
    "--tmp",
    `--chain=${chain}-dev`,
    "--wasm-execution=interpreted-i-know-what-i-do",
    "--rpc-port=9933"
  ];

  const child = spawn("../target/release/moonbeam", args, {
    detached: true,
    stdio: "ignore"
  });

  if (!child.pid) {
    throw new Error("Failed to start node process");
  }

  return child.pid;
}

async function waitForNode(): Promise<void> {
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

async function getMetadata(): Promise<string> {
  const response = await axios.post("http://localhost:9933", {
    id: "1",
    jsonrpc: "2.0",
    method: "state_getMetadata",
    params: []
  });
  return JSON.stringify(response.data);
}

async function main() {
  const runtimeChainSpec = process.argv[2];

  // Bump package version if parameter is provided
  if (runtimeChainSpec) {
    console.log(`Bump package version to 0.${runtimeChainSpec}.0`);
    execSync(`npm version --no-git-tag-version 0.${runtimeChainSpec}.0`);
  }

  // Check for moonbeam binary
  if (!existsSync("../target/release/moonbeam")) {
    console.error("Missing ../target/release/moonbeam binary");
    process.exit(1);
  }

  // Install dependencies
  execSync("pnpm install");

  // Get runtimes metadata
  for (const chain of CHAINS) {
    console.log(`Starting ${chain} node`);
    const pid = await startNode(chain);

    console.log("Waiting node...");
    await waitForNode();

    console.log(`Getting ${chain} metadata`);
    const metadata = await getMetadata();

    // Write metadata to file
    writeFileSync(`metadata-${chain}.json`, metadata);

    // Kill the node
    process.kill(pid);
    await new Promise((resolve) => setTimeout(resolve, 5000));
  }

  // Generate typescript api code
  console.log("Generating typescript api code...");
  execSync("pnpm generate:defs && pnpm generate:meta");

  // Hack: polkadot-js does not support XCM v5 yet, we need to manually change some types
  hackXcmV5Support();

  // Build the package
  console.log("Building package...");
  execSync("pnpm run build");
  console.log("Package built successfully!");

  console.log("Running format fix...");
  execSync("pnpm fmt:fix");
  console.log("Format fix completed!");
}

main().catch((error) => {
  console.error("Error:", error);
  process.exit(1);
});

import fs from "node:fs";
import { ChildProcessWithoutNullStreams, execSync, spawn } from "node:child_process";
import path from "node:path";

const CHAINS = ["moonbase", "moonriver", "moonbeam"];

const fetchMetadata = async (port: number = 9933) => {
  const maxRetries = 60;
  const sleepTime = 500;
  const url = `http://localhost:${port}`;
  const payload = {
    id: "1",
    jsonrpc: "2.0",
    method: "state_getMetadata",
    params: [],
  };

  for (let i = 0; i < maxRetries; i++) {
    try {
      const response = await fetch(url, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(payload),
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const data = await response.json();
      return data;
    } catch {
      console.log("Waiting for node to launch...");
      await new Promise((resolve) => setTimeout(resolve, sleepTime));
    }
  }
  console.log(`Error fetching metadata after ${(maxRetries * sleepTime) / 1000} seconds`);
  throw new Error("Error fetching metadata");
};

let nodes: { [key: string]: ChildProcessWithoutNullStreams } = {};

async function main() {
  const runtimeChainSpec = process.argv[2];
  const nodePath = path.join(process.cwd(), "..", "target", "release", "moonbeam");

  if (runtimeChainSpec) {
    console.log(`Bump package version to 0.${runtimeChainSpec}.0`);
    execSync(`npm version --no-git-tag-version 0.${runtimeChainSpec}.0`);
  }

  if (!fs.existsSync(nodePath)) {
    console.error("Moonbeam Node not found at path: ", nodePath);
    throw new Error("File not found");
  }

  for (const chain of CHAINS) {
    console.log(`Starting ${chain} node`);
    nodes[chain] = spawn(nodePath, [
      "--no-hardware-benchmarks",
      "--unsafe-force-node-key-generation",
      "--no-telemetry",
      "--no-prometheus",
      "--alice",
      "--tmp",
      `--chain=${chain}-dev`,
      "--wasm-execution=interpreted-i-know-what-i-do",
      "--rpc-port=9933",
    ]);

    console.log(`Getting ${chain} metadata`);
    try {
      const metadata = await fetchMetadata();
      fs.writeFileSync(`metadata-${chain}.json`, JSON.stringify(metadata, null, 2));
      console.log(`✅ Metadata for ${chain} written to metadata-${chain}.json`);
    } catch (error) {
      console.error(`❌ Error getting metadata for ${chain}:`, error);
    } finally {
      nodes[chain].kill();
      await new Promise((resolve) => setTimeout(resolve, 2)); // Wait 5 seconds between chains
    }
  }
}

process.on("SIGINT", () => {
  Object.values(nodes).forEach((node) => node.kill());
  process.exit();
});

main()
  .catch((error) => {
    console.error(error);
    process.exitCode = 1;
  })
  .finally(() => {
    Object.values(nodes).forEach((node) => node.kill());
  });

/**
 *  Script to launch 2 relay and 2 parachain nodes.
 *  It contains pre-registered versions to allow easy run using Docker.
 *
 *  ports can be given using --port-prefix xx (default 34) using the following rule:
 *  - relay 1 - p2p (p2p: XX000, rpcPort: XX001)
 *  - relay 2 - p2p (p2p: XX010, rpcPort: XX011)
 *  - para 1 - p2p (p2p: XX100, rpcPort: XX101)
 *  - para 2 - p2p (p2p: XX110, rpcPort: XX111)
 *
 */

import yargs from "yargs";
import * as fs from "node:fs";
import * as path from "path";
import * as child_process from "node:child_process";
import { killAll, run } from "polkadot-launch";

export async function getDockerBuildBinary(
  dockerImage: string,
  binaryPath: string
): Promise<string> {
  if (process.platform != "linux") {
    console.error(
      `docker binaries are only supported on linux. Use "local" config for compiled binaries`
    );
    process.exit(1);
  }
  child_process.execSync(`docker create --name moonbeam-tmp ${dockerImage} && \
      docker cp moonbeam-tmp:/moonbeam/moonbeam ${binaryPath} && \
      docker rm moonbeam-tmp`);
  return binaryPath;
}

export async function getGithubReleaseBinary(url: string, binaryPath: string): Promise<string> {
  child_process.execSync(`wget -q ${url}` + ` -O ${binaryPath}`);
  return binaryPath;
}

// Downloads the binary and return the filepath
export async function getMoonbeamBinary(binaryTag: string, binaryPath: string): Promise<string> {
  if (binaryTag.startsWith(`v`)) {
    return getGithubReleaseBinary(
      `https://github.com/moonbeam-foundation/moonbeam/releases/download/${binaryTag}/moonbeam`,
      binaryPath
    );
  } else if (binaryTag.startsWith(`sha`)) {
    return getDockerBuildBinary(`moonbeamfoundation/moonbeam:${binaryTag}`, binaryPath);
  } else if (/^[0-9]/g.test(binaryTag)) {
    // sha given without prefix
    return getDockerBuildBinary(`moonbeamfoundation/moonbeam:sha-${binaryTag}`, binaryPath);
  } else {
    const sha = child_process.execSync(`git rev-list -n 1 ${binaryTag}`).toString();
    return getDockerBuildBinary(`moonbeamfoundation/moonbeam:sha-${sha.slice(0, 8)}`, binaryPath);
  }
}

async function start() {
  const argv = yargs(process.argv.slice(2))
    .usage("Usage: npm run launch [args]")
    .version("1.0.0")
    .options({
      tag: {
        type: "string",
        describe: "<sha-xxxx> or <v0.xx.x>",
        demandOption: true,
      },
      "output-dir": {
        type: "string",
        alias: "o",
        describe: "folder to copy the binary to",
        default: ".",
      },
    })
    .help().argv;

  const binaryPath = path.join(argv["output-dir"], `/moonbeam-${argv.tag}`);
  console.log(`Downloading ${argv.tag}...`);
  await getMoonbeamBinary(argv.tag, binaryPath);
  child_process.execSync(`chmod uog+x ${binaryPath}`);
  console.log(`Copied binary ${argv.tag} to ${binaryPath}`);
}

start();

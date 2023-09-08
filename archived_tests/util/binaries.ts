import path from "path";
import fs from "fs";
import child_process from "child_process";

import { OVERRIDE_RUNTIME_PATH } from "./constants";

export const BINARY_DIRECTORY = process.env.BINARY_DIRECTORY || "binaries";
export const RUNTIME_DIRECTORY = process.env.RUNTIME_DIRECTORY || "runtimes";
export const SPECS_DIRECTORY = process.env.SPECS_DIRECTORY || "specs";

export async function getGithubReleaseBinary(url: string, binaryPath: string): Promise<string> {
  if (!fs.existsSync(binaryPath)) {
    console.log(`     Missing ${binaryPath} locally, downloading it...`);
    child_process.execSync(
      `mkdir -p ${path.dirname(binaryPath)} &&` +
        ` wget -q ${url}` +
        ` -O ${binaryPath} &&` +
        ` chmod u+x ${binaryPath}`
    );
    console.log(`${binaryPath} downloaded !`);
  }
  return binaryPath;
}

// Downloads the binary and return the filepath
export async function getMoonbeamReleaseBinary(binaryTag: string): Promise<string> {
  const binaryPath = path.join(BINARY_DIRECTORY, `moonbeam-${binaryTag}`);
  return getGithubReleaseBinary(
    `https://github.com/moonbeam-foundation/moonbeam/releases/download/${binaryTag}/moonbeam`,
    binaryPath
  );
}

export async function getPolkadotReleaseBinary(binaryTag: string): Promise<string> {
  const binaryPath = path.join(BINARY_DIRECTORY, `polkadot-${binaryTag}`);
  return getGithubReleaseBinary(
    `https://github.com/paritytech/polkadot/releases/download/${binaryTag}/polkadot`,
    binaryPath
  );
}

export async function getTagSha8(binaryTag: string): Promise<string> {
  const sha = child_process.execSync(`git rev-list -1 ${binaryTag}`).toString();
  if (!sha) {
    throw new Error(`Invalid runtime tag ${binaryTag}`);
    return;
  }
  return sha.slice(0, 8);
}

export async function getMoonbeamDockerBinary(binaryTag: string): Promise<string> {
  const sha8 = await getTagSha8(binaryTag);
  const binaryPath = path.join(BINARY_DIRECTORY, `moonbeam-${sha8}`);
  if (!fs.existsSync(binaryPath)) {
    if (process.platform != "linux") {
      console.error(`docker binaries are only supported on linux.`);
      process.exit(1);
    }
    const dockerImage = `moonbeamfoundation/moonbeam:sha-${sha8}`;

    console.log(`     Missing ${binaryPath} locally, downloading it...`);
    child_process.execSync(`mkdir -p ${path.dirname(binaryPath)} && \
          docker create --pull always --name moonbeam-tmp ${dockerImage} && \
          docker cp moonbeam-tmp:/moonbeam/moonbeam ${binaryPath} && \
          docker rm moonbeam-tmp`);
    console.log(`${binaryPath} downloaded !`);
  }
  return binaryPath;
}

// Downloads the runtime and return the filepath
export async function getRuntimeWasm(
  runtimeName: "moonbase" | "moonriver" | "moonbeam",
  runtimeTag: "local" | string
): Promise<string> {
  const runtimePath = path.join(RUNTIME_DIRECTORY, `${runtimeName}-${runtimeTag}.wasm`);

  if (!fs.existsSync(RUNTIME_DIRECTORY)) {
    fs.mkdirSync(RUNTIME_DIRECTORY, { recursive: true });
  }

  if (runtimeTag == "local") {
    const builtRuntimePath = path.join(
      OVERRIDE_RUNTIME_PATH || `../target/release/wbuild/${runtimeName}-runtime/`,
      `${runtimeName}_runtime.compact.compressed.wasm`
    );

    const code = fs.readFileSync(builtRuntimePath);
    fs.writeFileSync(runtimePath, `0x${code.toString("hex")}`);
  } else if (!fs.existsSync(runtimePath)) {
    console.log(`     Missing ${runtimePath} locally, downloading it...`);
    child_process.execSync(
      `mkdir -p ${path.dirname(runtimePath)} && ` +
        `wget -q https://github.com/moonbeam-foundation/moonbeam/releases/` +
        `download/${runtimeTag}/${runtimeName}-${runtimeTag}.wasm ` +
        `-O ${runtimePath}.bin`
    );
    const code = fs.readFileSync(`${runtimePath}.bin`);
    fs.writeFileSync(runtimePath, `0x${code.toString("hex")}`);
    console.log(`${runtimePath} downloaded !`);
  }
  return runtimePath;
}

export async function getPlainSpecsFromTag(
  runtimeName: "moonbase-local" | "moonriver-local" | "moonbeam-local",
  tag: string
) {
  const binaryPath = await getMoonbeamDockerBinary(tag);
  return generateSpecs(binaryPath, runtimeName, false);
}

export async function getRawSpecsFromTag(
  runtimeName: "moonbase-local" | "moonriver-local" | "moonbeam-local",
  tag: string
) {
  const binaryPath = await getMoonbeamDockerBinary(tag);
  return generateSpecs(binaryPath, runtimeName, true);
}

async function generateSpecs(
  binaryPath: string,
  runtimeName: "moonbase-local" | "moonriver-local" | "moonbeam-local",
  raw: boolean
) {
  const specPath = path.join(SPECS_DIRECTORY, `${runtimeName}-${raw ? "raw" : "plain"}-specs.json`);
  child_process.execSync(
    `mkdir -p ${path.dirname(specPath)} && ` +
      `${binaryPath} build-spec --chain ${runtimeName} ` +
      `${raw ? "--raw" : ""} --disable-default-bootnode > ${specPath}`
  );
  return specPath;
}

export async function generatePlainSpecs(
  binaryPath: string,
  runtimeName: "moonbase-local" | "moonriver-local" | "moonbeam-local"
) {
  return generateSpecs(binaryPath, runtimeName, false);
}

export async function generateRawSpecs(
  binaryPath: string,
  runtimeName: "moonbase-local" | "moonriver-local" | "moonbeam-local"
) {
  return generateSpecs(binaryPath, runtimeName, true);
}

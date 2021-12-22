import { execSync } from "child_process";
import { Octokit } from "octokit";
import { readFileSync } from "fs";
import yargs from "yargs";
import path from "path";
import { getCommitAndLabels, getCompareLink } from "./github-utils";
import { blake2AsHex } from "@polkadot/util-crypto";

const RUNTIME_CHANGES_LABEL = "B7-runtimenoteworthy";
// `ParachainSystem` is pallet index 6. `authorize_upgrade` is extrinsic index 2.
const MOONBASE_PREFIX_PARACHAINSYSTEM_AUTHORIZE_UPGRADE = "0x0602";
// `ParachainSystem` is pallet index 1. `authorize_upgrade` is extrinsic index 2.
const MOONRIVER_PREFIX_PARACHAINSYSTEM_AUTHORIZE_UPGRADE = "0x0102";
// `ParachainSystem` is pallet index 1. `authorize_upgrade` is extrinsic index 2.
const MOONBEAM_PREFIX_PARACHAINSYSTEM_AUTHORIZE_UPGRADE = "0x0102";

function capitalize(s) {
  return s[0].toUpperCase() + s.slice(1);
}

function getRuntimeInfo(srtoolReportFolder: string, runtimeName: string) {
  const specVersion = execSync(
    `cat ../runtime/${runtimeName}/src/lib.rs | grep 'spec_version: [0-9]*' | tail -1`
  ).toString();
  return {
    name: runtimeName,
    version: /:\s?([0-9A-z\-]*)/.exec(specVersion)[1],
    srtool: JSON.parse(
      readFileSync(path.join(srtoolReportFolder, `./${runtimeName}-srtool-digest.json`)).toString()
    ),
  };
}

// Srtool expects the pallet parachain_system to be at index 1. However, in the case of moonbase,
// the pallet parachain_system is at index 6, so we have to recalculate the hash of the
// authorizeUpgrade call in the case of moonbase by hand.
function authorizeUpgradeHash(runtimeName: string, srtool: any): string {
  if (runtimeName == "moonbase") {
    return blake2AsHex(
      MOONBASE_PREFIX_PARACHAINSYSTEM_AUTHORIZE_UPGRADE +
        srtool.runtimes.compressed.blake2_256.substr(2) // remove "0x" prefix
    );
  } else if (runtimeName == "moonriver") {
    return blake2AsHex(
      MOONRIVER_PREFIX_PARACHAINSYSTEM_AUTHORIZE_UPGRADE +
        srtool.runtimes.compressed.blake2_256.substr(2) // remove "0x" prefix
    );
  } else {
    return blake2AsHex(
      MOONBEAM_PREFIX_PARACHAINSYSTEM_AUTHORIZE_UPGRADE +
        srtool.runtimes.compressed.blake2_256.substr(2) // remove "0x" prefix
    );
  }
}

/*async function main() {
  console.log("moonbase: " + blake2AsHex(MOONBASE_PREFIX_PARACHAINSYSTEM_AUTHORIZE_UPGRADE + "4f92a8cddee4237ae3ef06b59a6dfe4a59cb6dfd4017e13fa51a2c8cba3813ca"));
  console.log("moonriver: " + blake2AsHex(MOONRIVER_PREFIX_PARACHAINSYSTEM_AUTHORIZE_UPGRADE + "8183ebaf5c12edb993752c21af6cf4670cc67e1a62a920bdbf1ff9f54dad5fc2"));
  console.log("moonbeam: " + blake2AsHex(MOONBEAM_PREFIX_PARACHAINSYSTEM_AUTHORIZE_UPGRADE + "e01a1363423212a82d8a50ef47da36d6eb53bf651d9ed652ec330c1157ec7079"));
}*/

async function main() {
  const argv = yargs(process.argv.slice(2))
    .usage("Usage: npm run ts-node github/generate-release-body.ts [args]")
    .version("1.0.0")
    .options({
      "srtool-report-folder": {
        type: "string",
        describe: "folder which contains <runtime>-srtool-digest.json",
        required: true,
      },
      from: {
        type: "string",
        describe: "previous tag to retrieve commits from",
        required: true,
      },
      to: {
        type: "string",
        describe: "current tag to draft",
        required: true,
      },
    })
    .demandOption(["srtool-report-folder", "from", "to"])
    .help().argv;

  const octokit = new Octokit({
    auth: process.env.GITHUB_TOKEN || undefined,
  });

  const previousTag = argv.from;
  const newTag = argv.to;

  const runtimes = ["moonbase", "moonriver", "moonbeam"].map((runtimeName) =>
    getRuntimeInfo(argv["srtool-report-folder"], runtimeName)
  );

  const moduleLinks = ["substrate", "polkadot", "cumulus", "frontier"].map((repoName) => ({
    name: repoName,
    link: getCompareLink(repoName, previousTag, newTag),
  }));

  const { prByLabels } = await getCommitAndLabels(
    octokit,
    "purestake",
    "moonbeam",
    previousTag,
    newTag
  );
  const filteredPr = prByLabels[RUNTIME_CHANGES_LABEL] || [];

  //

  const template = `${
    runtimes.length > 0
      ? `## Runtimes

${runtimes
  .map(
    (runtime) => `### ${capitalize(runtime.name)}
\`\`\`
✨ spec_version                : ${runtime.version}
🏋 size                        : ${runtime.srtool.runtimes.compressed.size}
#️⃣ sha256                      : ${runtime.srtool.runtimes.compressed.sha256}
#️⃣ blake2-256                  : ${runtime.srtool.runtimes.compressed.blake2_256}
🗳️ proposal (authorizeUpgrade) : ${authorizeUpgradeHash(runtime.name, runtime.srtool)}
\`\`\``
  )
  .join(`\n\n`)}
`
      : ""
  }

## Build information

WASM runtime built using \`${runtimes[0]?.srtool.info.rustc}\`

## Changes

${filteredPr.map((pr) => `* ${pr.title} (#${pr.number})`).join("\n")}

## Dependency changes

Moonbeam: https://github.com/PureStake/moonbeam/compare/${previousTag}...${newTag}
${moduleLinks.map((modules) => `${capitalize(modules.name)}: ${modules.link}`).join("\n")}
`;
  console.log(template);
}

main();

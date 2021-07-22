import { execSync } from "child_process";
import { Octokit } from "octokit";
import { readFileSync } from "fs";
import yargs from "yargs";
import path from "path";
import { getCommitAndLabels, getCompareLink } from "./github-utils";

const RUNTIME_CHANGES_LABEL = "B7-runtimenoteworthy";

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

  const runtimes = ["moonbase", "moonshadow", "moonriver", "moonbeam"].map((runtimeName) =>
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

  const template = `${
    runtimes.length > 0
      ? `## Runtimes

${runtimes
  .map(
    (runtime) => `### ${capitalize(runtime.name)}

* spec_version: ${runtime.version}
* sha256: ${runtime.srtool.runtimes.compact.sha256}
* size: ${runtime.srtool.runtimes.compact.size}
* proposal: ${runtime.srtool.runtimes.compact.prop}`
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
